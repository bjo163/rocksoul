use std::{
    io::{self, BufRead, Write},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use reqwest::Client;

use crate::{
    commands,
    core::{
        CancellationToken, ReflectionOutcome, TurnBudget, TurnEvent, TurnMeter, TurnPlan,
        TurnReflection, TurnRequest, TurnState,
    },
    model::{ChatMessage, ModelConfig},
};

static TURN_SEQUENCE: AtomicU64 = AtomicU64::new(1);

const SYSTEM_PROMPT: &str = "You are Rocksoul, an experimental CLI intelligence runtime. Be accurate and explicit about uncertainty. You have no tools and cannot inspect files, run commands, or modify systems. Never claim that an action was performed unless the runtime actually performed it.";
const PLANNER_PROMPT: &str = "You are Rocksoul's planner. Return ONLY JSON with this shape: {\"objective\":string,\"steps\":[{\"id\":string,\"description\":string}],\"acceptance_checks\":[{\"id\":string,\"description\":string,\"required\":boolean}]}. Create 1-3 small ordered reasoning steps and at least one required acceptance check. No markdown or extra prose. Plans are proposals, not executable commands.";
const REFLECTOR_PROMPT: &str = "You are the reflector. Review the plan, required checks, and supplied model outputs. Do not claim external actions or facts were verified. Return ONLY JSON with outcome (succeeded, needs_retry, or failed), summary, and issues (array of strings). Use succeeded only when every required check is supported by the outputs; otherwise list concrete unresolved issues. No markdown.";
const MAX_HISTORY_MESSAGES: usize = 41;
const MAX_PLAN_STEPS: usize = 3;

pub async fn run() -> Result<()> {
    let model = ModelConfig::from_env();
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(120))
        .build()
        .context("could not initialize the HTTP client")?;
    let mut messages = vec![ChatMessage::new("system", SYSTEM_PROMPT)];

    println!("Rocksoul v{} — Seed CLI", env!("CARGO_PKG_VERSION"));
    println!("Type :help for commands. Type :exit to leave.");
    if model.is_none() {
        println!("No model configured. Set ROCKSOUL_MODEL to enable chat.");
    }

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut input = String::new();
    loop {
        print!("rocksoul> ");
        io::stdout().flush()?;
        input.clear();
        if reader.read_line(&mut input)? == 0 {
            println!();
            break;
        }
        let prompt = input.trim();
        if prompt.is_empty() {
            continue;
        }
        match prompt.to_ascii_lowercase().as_str() {
            ":help" | "help" | "?" => print_help(),
            ":status" | "status" => commands::status()?,
            ":init" | "init" => commands::init()?,
            ":clear" | "clear" => {
                messages.clear();
                messages.push(ChatMessage::new("system", SYSTEM_PROMPT));
                println!("Conversation cleared.");
            }
            ":exit" | ":quit" | "exit" | "quit" => {
                println!("Rocksoul session ended.");
                break;
            }
            _ => {
                let Some(model) = &model else {
                    println!("Chat is unavailable until ROCKSOUL_MODEL is configured.");
                    println!("See README.md for local and hosted endpoint examples.");
                    continue;
                };
                messages.push(ChatMessage::new("user", prompt));
                match run_turn(model, &client, prompt, &messages).await {
                    Ok(answer) => {
                        println!("\n{answer}\n");
                        messages.push(ChatMessage::new("assistant", answer));
                        trim_history(&mut messages);
                    }
                    Err(error) => {
                        messages.pop();
                        eprintln!("Turn failed: {error:#}");
                    }
                }
            }
        }
    }
    Ok(())
}

/// One planning call followed by at most three bounded model-only execution calls.
async fn run_turn(
    model: &ModelConfig,
    client: &Client,
    input: &str,
    history: &[ChatMessage],
) -> Result<String> {
    let request = TurnRequest {
        id: format!("turn-{}", TURN_SEQUENCE.fetch_add(1, Ordering::Relaxed)),
        input: input.to_owned(),
        budget: TurnBudget::default(),
    };
    request.validate()?;
    let cancellation = CancellationToken::default();
    let mut meter = TurnMeter::new(request.budget)?;
    let mut state = TurnState::new(request.id, 0);
    state.apply(TurnEvent::Start)?;

    meter.record_model_call(&cancellation)?;
    let planning_messages = [
        ChatMessage::new("system", PLANNER_PROMPT),
        ChatMessage::new("user", input),
    ];
    let planning = model.complete(client, &planning_messages).await?;
    meter.check_time_budget(&cancellation)?;
    record_usage(&mut meter, planning.output_tokens)?;
    let plan = parse_plan(&planning.content)?;
    plan.validate()?;
    if plan.steps.len() > MAX_PLAN_STEPS {
        bail!(
            "planner returned too many steps: {} (maximum {})",
            plan.steps.len(),
            MAX_PLAN_STEPS
        );
    }
    state.apply(TurnEvent::PlanAccepted)?;

    let mut execution = history.to_vec();
    execution.push(ChatMessage::new(
        "system",
        format!("Accepted objective: {}. Work through the supplied steps in order. You have no tools. Never claim external actions.", plan.objective),
    ));
    let mut outputs = Vec::with_capacity(plan.steps.len());

    for (index, step) in plan.steps.iter().enumerate() {
        meter.record_model_call(&cancellation)?;
        execution.push(ChatMessage::new(
            "user",
            format!(
                "Plan step {}/{} [{}]: {}. Provide the result for this step only.",
                index + 1,
                plan.steps.len(),
                step.id,
                step.description
            ),
        ));
        let completion = model.complete(client, &execution).await?;
        meter.check_time_budget(&cancellation)?;
        record_usage(&mut meter, completion.output_tokens)?;
        execution.push(ChatMessage::new("assistant", completion.content.clone()));
        outputs.push(format!(
            "Step {} — {}\n{}",
            index + 1,
            step.id,
            completion.content
        ));
    }

    state.apply(TurnEvent::ActionCompleted)?;
    // Reject missing outputs deterministically before model-based review.
    if outputs.len() != plan.steps.len() || outputs.iter().any(|item| item.trim().is_empty()) {
        state.apply(TurnEvent::VerificationRejected)?;
        bail!("one or more execution steps did not produce a usable result");
    }

    meter.record_model_call(&cancellation)?;
    let review_payload = serde_json::json!({
        "objective": plan.objective,
        "steps": plan.steps,
        "acceptance_checks": plan.acceptance_checks,
        "execution_outputs": outputs,
    });
    let reflection_messages = [
        ChatMessage::new("system", REFLECTOR_PROMPT),
        ChatMessage::new("user", serde_json::to_string(&review_payload)?),
    ];
    let review = model.complete(client, &reflection_messages).await?;
    meter.check_time_budget(&cancellation)?;
    record_usage(&mut meter, review.output_tokens)?;
    let reflection = parse_reflection(&review.content)?;
    reflection.validate()?;

    match reflection.outcome {
        ReflectionOutcome::Succeeded => state.apply(TurnEvent::VerificationPassed)?,
        ReflectionOutcome::NeedsRetry | ReflectionOutcome::Failed => {
            state.apply(TurnEvent::VerificationRejected)?;
            let issues = if reflection.issues.is_empty() {
                "no specific issue was supplied".to_owned()
            } else {
                reflection.issues.join("; ")
            };
            bail!("reflector did not approve this turn: {} ({issues})", reflection.summary);
        }
    }
    debug_assert_eq!(state.phase, crate::core::TurnPhase::Completed);
    Ok(format!("{}\n\nReflection: {}", outputs.join("\n\n"), reflection.summary))
}

fn parse_plan(raw: &str) -> Result<TurnPlan> {
    let trimmed = raw.trim();
    let json = if let Some(rest) = trimmed.strip_prefix("```json") {
        rest.trim()
            .strip_suffix("```")
            .unwrap_or(rest.trim())
            .trim()
    } else if let Some(rest) = trimmed.strip_prefix("```") {
        rest.trim()
            .strip_suffix("```")
            .unwrap_or(rest.trim())
            .trim()
    } else {
        trimmed
    };
    serde_json::from_str(json).context("planner returned invalid structured JSON")
}

fn parse_reflection(raw: &str) -> Result<TurnReflection> {
    let trimmed = raw.trim();
    let json = if let Some(rest) = trimmed.strip_prefix("```json") {
        rest.trim()
            .strip_suffix("```")
            .unwrap_or(rest.trim())
            .trim()
    } else if let Some(rest) = trimmed.strip_prefix("```") {
        rest.trim()
            .strip_suffix("```")
            .unwrap_or(rest.trim())
            .trim()
    } else {
        trimmed
    };
    serde_json::from_str(json).context("reflector returned invalid structured JSON")
}

fn record_usage(meter: &mut TurnMeter, tokens: Option<u32>) -> Result<()> {
    if let Some(tokens) = tokens {
        meter.record_output_tokens(tokens)?;
    }
    Ok(())
}

fn trim_history(messages: &mut Vec<ChatMessage>) {
    if messages.len() > MAX_HISTORY_MESSAGES {
        let excess = messages.len() - MAX_HISTORY_MESSAGES;
        messages.drain(1..=excess);
    }
}

fn print_help() {
    println!("Rocksoul CLI commands:");
    println!("  :help    Show this help");
    println!("  :status  Show runtime and model status");
    println!("  :init    Initialize local directories and config");
    println!("  :clear   Clear the current conversation");
    println!("  :exit    Exit Rocksoul");
    println!();
    println!("Any other input is sent to the configured chat model.");
}

#[cfg(test)]
mod tests {
    use super::{parse_plan, parse_reflection};

    #[test]
    fn parses_valid_plan_json() {
        let raw = r#"{"objective":"Explain a topic","steps":[{"id":"s1","description":"Define the topic"}],"acceptance_checks":[{"id":"c1","description":"A definition is included","required":true}]}"#;
        let plan = parse_plan(raw).unwrap();
        assert!(plan.validate().is_ok());
        assert_eq!(plan.steps.len(), 1);
    }

    #[test]
    fn parses_fenced_plan_json() {
        let json = r#"{"objective":"Explain","steps":[{"id":"s1","description":"Define"}],"acceptance_checks":[{"id":"c1","description":"A definition is included","required":true}]}"#;
        let wrapped = format!("{}\n{}\n{}", "\x60\x60\x60json", json, "\x60\x60\x60");
        let plan = parse_plan(&wrapped).unwrap();
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn parses_valid_reflection_json() {
        let raw = r#"{"outcome":"succeeded","summary":"All required checks are supported","issues":[]}"#;
        let reflection = parse_reflection(raw).unwrap();
        assert!(reflection.validate().is_ok());
    }

    #[test]
    fn rejects_malformed_reflection_json() {
        assert!(parse_reflection("not json").is_err());
    }

    #[test]
    fn rejects_successful_reflection_with_issues() {
        let raw = r#"{"outcome":"succeeded","summary":"Looks good","issues":["Missing evidence"]}"#;
        let reflection = parse_reflection(raw).unwrap();
        assert!(reflection.validate().is_err());
    }

    #[test]
    fn rejects_malformed_plan_json() {
        assert!(parse_plan("not json").is_err());
    }
}
