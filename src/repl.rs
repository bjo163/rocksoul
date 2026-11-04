use std::{
    io::{self, Write},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use reqwest::Client;

use crate::{
    cli::{Command, WorkspaceCommand},
    commands,
    core::{
        CancellationToken, ReflectionOutcome, TurnBudget, TurnEvent, TurnMeter, TurnPlan,
        TurnReflection, TurnRequest, TurnState,
    },
    model::{ChatMessage, ModelConfig},
    workspace_artifact::WorkspaceArtifactProposal,
};

const SYSTEM_PROMPT: &str = "You are Rocksoul, an experimental CLI intelligence runtime. Be accurate and explicit about uncertainty. You cannot inspect files or run commands. You may propose one narrow workspace text artifact only when the user explicitly asks; the runtime must show it and obtain exact human approval before writing. Never claim an action was performed unless the runtime actually performed and verified it.";
const PLANNER_PROMPT: &str = "You are Rocksoul's planner. Return ONLY JSON with this shape: {\"objective\":string,\"steps\":[{\"id\":string,\"description\":string}],\"acceptance_checks\":[{\"id\":string,\"description\":string,\"required\":boolean}],\"workspace_artifact\":null|{\"filename\":string,\"content\":string}}. Create 1-3 small ordered reasoning steps and at least one required acceptance check. Set workspace_artifact only when the user explicitly asks to create a text file in the Rocksoul workspace; otherwise set it to null. Only propose a flat ASCII .txt filename and content. This is a proposal only; the runtime will show it and require exact human approval before writing. No markdown or extra prose.";
const REFLECTOR_PROMPT: &str = "You are the reflector. Review the plan, required checks, and supplied model outputs. Do not claim external actions or facts were verified. Return ONLY JSON with outcome (succeeded, needs_retry, or failed), summary, and issues (array of strings). Use succeeded only when every required check is supported by the outputs; otherwise list concrete unresolved issues. No markdown.";
const MAX_HISTORY_MESSAGES: usize = 41;
const MAX_PLAN_STEPS: usize = 3;

#[derive(Clone, Debug, serde::Deserialize)]
struct WorkspaceArtifactIntent {
    filename: String,
    content: String,
}

#[derive(serde::Deserialize)]
struct PlannerResponse {
    #[serde(flatten)]
    plan: TurnPlan,
    #[serde(default)]
    workspace_artifact: Option<WorkspaceArtifactIntent>,
}

struct TurnResult {
    answer: String,
    elapsed_ms: u64,
    model_calls: u32,
    reported_output_tokens: u32,
    usage_complete: bool,
    workspace_artifact: Option<WorkspaceArtifactIntent>,
}

pub async fn run() -> Result<()> {
    let model = ModelConfig::from_env();
    // One planner + up to three step calls + one reflector.
    let turn_budget = TurnBudget {
        max_model_calls: 5,
        max_output_tokens: 16_000,
        timeout_ms: 120_000,
    };
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_millis(turn_budget.timeout_ms))
        .build()
        .context("could not initialize the HTTP client")?;

    let mut messages = vec![ChatMessage::new("system", SYSTEM_PROMPT)];
    let mut turn_sequence = 0_u64;

    println!("Rocksoul v{} — Seed CLI", env!("CARGO_PKG_VERSION"));
    println!("Type :help for commands. Type :exit to leave.");
    if model.is_none() {
        println!("No model configured. Set ROCKSOUL_MODEL to enable chat.");
    }

    let stdin = io::stdin();
    let mut input = String::new();
    loop {
        print!("rocksoul> ");
        io::stdout().flush()?;
        input.clear();
        if stdin.read_line(&mut input)? == 0 {
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
            ":rid" | "rid" => commands::rid_status()?,
            ":memory" | "memory" => commands::memory_status()?,
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

                turn_sequence = turn_sequence.saturating_add(1);
                let request = TurnRequest {
                    id: format!("turn-{turn_sequence}"),
                    input: prompt.to_owned(),
                    budget: turn_budget,
                };
                request.validate()?;
                messages.push(ChatMessage::new("user", prompt));

                let cancellation = CancellationToken::default();
                let history_snapshot = messages.clone();
                let turn = run_turn(
                    model,
                    &client,
                    request,
                    &history_snapshot,
                    cancellation.clone(),
                );
                tokio::pin!(turn);
                let result = tokio::select! {
                    result = &mut turn => result,
                    signal = tokio::signal::ctrl_c() => {
                        if let Err(error) = signal {
                            eprintln!("Could not register Ctrl+C handler: {error}");
                        }
                        cancellation.cancel();
                        turn.await
                    }
                };
                match result {
                    Ok(result) => {
                        println!("\n{}\n", result.answer);
                        if let Some(artifact) = result.workspace_artifact {
                            let command = Command::Workspace {
                                command: WorkspaceCommand::Write {
                                    filename: artifact.filename,
                                    content: artifact.content,
                                },
                            };
                            if let Err(error) = commands::run(command) {
                                eprintln!("Workspace proposal was not completed: {error:#}");
                            }
                        }
                        let token_label = if result.usage_complete {
                            format!("{} reported output tokens", result.reported_output_tokens)
                        } else {
                            format!(
                                "{} reported output tokens (partial; provider omitted usage)",
                                result.reported_output_tokens
                            )
                        };
                        println!(
                            "[turn: {} ms · {} model calls · {}]",
                            result.elapsed_ms, result.model_calls, token_label
                        );
                        messages.push(ChatMessage::new("assistant", result.answer));
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

/// One planning call, up to three bounded model-only reasoning calls, and a reflector call.
/// The planner may return a narrow artifact proposal; dispatch still requires exact human approval.
async fn run_turn(
    model: &ModelConfig,
    client: &Client,
    request: TurnRequest,
    history: &[ChatMessage],
    cancellation: CancellationToken,
) -> Result<TurnResult> {
    request.validate()?;
    let mut meter = TurnMeter::new(request.budget)?;
    let mut state = TurnState::new(request.id, 0);
    state.apply(TurnEvent::Start)?;

    let planning_messages = [
        ChatMessage::new("system", PLANNER_PROMPT),
        ChatMessage::new("user", request.input),
    ];
    let planning = model
        .complete(
            client,
            &planning_messages,
            request.budget,
            &mut meter,
            &cancellation,
        )
        .await?;
    meter.check_time_budget(&cancellation)?;
    let mut usage_complete = planning.output_tokens.is_some();
    let planner_response = parse_plan(&planning.content)?;
    let plan = planner_response.plan;
    let workspace_artifact = planner_response.workspace_artifact;
    plan.validate()?;
    if let Some(artifact) = &workspace_artifact {
        WorkspaceArtifactProposal {
            filename: artifact.filename.clone(),
            content: artifact.content.clone(),
        }
        .validate()?;
    }
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
        format!(
            "Accepted objective: {}. Work through the supplied steps in order. You have no tools. Never claim external actions.",
            plan.objective
        ),
    ));
    let mut outputs = Vec::with_capacity(plan.steps.len());

    for (index, step) in plan.steps.iter().enumerate() {
        meter.check_time_budget(&cancellation)?;
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
        let completion = model
            .complete(
                client,
                &execution,
                request.budget,
                &mut meter,
                &cancellation,
            )
            .await?;
        meter.check_time_budget(&cancellation)?;
        usage_complete &= completion.output_tokens.is_some();
        execution.push(ChatMessage::new("assistant", completion.content.clone()));
        outputs.push(format!(
            "Step {} — {}\n{}",
            index + 1,
            step.id,
            completion.content
        ));
    }

    state.apply(TurnEvent::ActionCompleted)?;
    if outputs.len() != plan.steps.len() || outputs.iter().any(|item| item.trim().is_empty()) {
        state.apply(TurnEvent::VerificationRejected)?;
        bail!("one or more execution steps did not produce a usable result");
    }

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
    let review = model
        .complete(
            client,
            &reflection_messages,
            request.budget,
            &mut meter,
            &cancellation,
        )
        .await?;
    meter.check_time_budget(&cancellation)?;
    usage_complete &= review.output_tokens.is_some();
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
            bail!(
                "reflector did not approve this turn: {} ({issues})",
                reflection.summary
            );
        }
    }

    debug_assert_eq!(state.phase, crate::core::TurnPhase::Completed);
    Ok(TurnResult {
        answer: format!(
            "Objective: {}\n\n{}\n\nReflection: {}",
            plan.objective,
            outputs.join("\n\n"),
            reflection.summary
        ),
        elapsed_ms: meter.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
        model_calls: meter.model_calls(),
        reported_output_tokens: meter.output_tokens(),
        usage_complete,
        workspace_artifact,
    })
}

fn parse_plan(raw: &str) -> Result<PlannerResponse> {
    let trimmed = raw.trim();
    let json = strip_json_fence(trimmed);
    serde_json::from_str(json).context("planner returned invalid structured JSON")
}

fn parse_reflection(raw: &str) -> Result<TurnReflection> {
    let trimmed = raw.trim();
    let json = strip_json_fence(trimmed);
    serde_json::from_str(json).context("reflector returned invalid structured JSON")
}

fn strip_json_fence(raw: &str) -> &str {
    let body = raw
        .strip_prefix("```json")
        .or_else(|| raw.strip_prefix("```"))
        .map(str::trim)
        .unwrap_or(raw);
    body.strip_suffix("```").map(str::trim).unwrap_or(body)
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
    println!("  :rid     Show Ruh Information (age, level, XP, stat coverage)");
    println!("  :memory  Show explicit local memory records");
    println!("  :clear   Clear the current conversation");
    println!("  :exit    Exit Rocksoul");
    println!();
    println!(
        "Each chat turn uses a bounded planner, up to three model-only steps, and a reflector."
    );
}

#[cfg(test)]
mod tests {
    use crate::workspace_artifact::WorkspaceArtifactProposal;

    use super::{parse_plan, parse_reflection};

    #[test]
    fn parses_valid_plan_json() {
        let raw = r#"{"objective":"Explain a topic","steps":[{"id":"s1","description":"Define the topic"}],"acceptance_checks":[{"id":"c1","description":"A definition is included","required":true}]}"#;
        let plan = parse_plan(raw).unwrap();
        assert!(plan.plan.validate().is_ok());
        assert_eq!(plan.plan.steps.len(), 1);
    }

    #[test]
    fn parses_fenced_plan_json() {
        let json = r#"{"objective":"Explain","steps":[{"id":"s1","description":"Define"}],"acceptance_checks":[{"id":"c1","description":"A definition is included","required":true}]}"#;
        let wrapped = format!("\x60\x60\x60json\n{json}\n\x60\x60\x60");
        let plan = parse_plan(&wrapped).unwrap();
        assert!(plan.plan.validate().is_ok());
    }

    #[test]
    fn parses_optional_workspace_artifact_intent() {
        let raw = r#"{"objective":"Create a note","steps":[{"id":"s1","description":"Prepare the note"}],"acceptance_checks":[{"id":"c1","description":"The note is ready","required":true}],"workspace_artifact":{"filename":"note.txt","content":"hello"}}"#;
        let response = parse_plan(raw).unwrap();
        let artifact = response.workspace_artifact.unwrap();
        assert_eq!(artifact.filename, "note.txt");
        assert_eq!(artifact.content, "hello");
    }

    #[test]
    fn rejects_unsafe_planner_workspace_artifact_intent() {
        let raw = r#"{"objective":"Create a note","steps":[{"id":"s1","description":"Prepare the note"}],"acceptance_checks":[{"id":"c1","description":"The note is ready","required":true}],"workspace_artifact":{"filename":"../escape.txt","content":"hello"}}"#;
        let response = parse_plan(raw).unwrap();
        let artifact = response.workspace_artifact.unwrap();
        assert!(
            WorkspaceArtifactProposal {
                filename: artifact.filename,
                content: artifact.content,
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn parses_valid_reflection_json() {
        let raw =
            r#"{"outcome":"succeeded","summary":"All required checks are supported","issues":[]}"#;
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
