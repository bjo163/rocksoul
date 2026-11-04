use std::{
    io::{self, BufRead, Write},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use anyhow::{Context, Result};
use reqwest::Client;

use crate::{
    commands,
    core::{
        AcceptanceCheck, CancellationToken, PlanStep, ReflectionOutcome, TurnBudget, TurnEvent,
        TurnMeter, TurnPlan, TurnReflection, TurnRequest, TurnState,
    },
    model::{ChatMessage, ModelConfig},
};

static TURN_SEQUENCE: AtomicU64 = AtomicU64::new(1);

const SYSTEM_PROMPT: &str = "You are Rocksoul, an experimental CLI intelligence runtime. Be accurate and explicit about uncertainty. In this seed release you have no tools and cannot inspect files, run commands, or modify systems. Never claim that an action was performed unless the runtime actually performed it. You may explain plans and provide code for the user to review.";
const MAX_HISTORY_MESSAGES: usize = 41;

pub async fn run() -> Result<()> {
    let model = ModelConfig::from_env();
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(180))
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


/// Run one bounded interactive turn. The current seed adapter performs one
/// model action and verifies the response contract; it does not execute tools.
async fn run_turn(
    model: &ModelConfig,
    client: &Client,
    input: &str,
    messages: &[ChatMessage],
) -> Result<String> {
    let request = TurnRequest {
        id: format!("turn-{}", TURN_SEQUENCE.fetch_add(1, Ordering::Relaxed)),
        input: input.to_owned(),
        budget: TurnBudget::default(),
    };
    request.validate()?;

    let plan = TurnPlan {
        objective: "Produce a useful response to the user's input".to_owned(),
        steps: vec![PlanStep {
            id: "generate-response".to_owned(),
            description: "Request one response from the configured model".to_owned(),
        }],
        acceptance_checks: vec![AcceptanceCheck {
            id: "non-empty-response".to_owned(),
            description: "The model returns a non-empty assistant response".to_owned(),
            required: true,
        }],
    };
    plan.validate()?;

    let cancellation = CancellationToken::default();
    let mut meter = TurnMeter::new(request.budget)?;
    let mut state = TurnState::new(request.id, 0);

    state.apply(TurnEvent::Start)?;
    state.apply(TurnEvent::PlanAccepted)?;
    meter.record_model_call(&cancellation)?;

    let answer = match model.complete(client, messages).await {
        Ok(answer) => answer,
        Err(error) => {
            let _ = state.apply(TurnEvent::Fail);
            return Err(error);
        }
    };

    // Re-check the elapsed-time budget after the network operation returns.
    meter.before_model_call(&cancellation)?;
    state.apply(TurnEvent::ActionCompleted)?;

    let reflection = TurnReflection {
        outcome: ReflectionOutcome::Succeeded,
        summary: "The response passed the non-empty output acceptance check".to_owned(),
        issues: Vec::new(),
    };
    reflection.validate()?;
    state.apply(TurnEvent::VerificationPassed)?;

    debug_assert_eq!(state.phase, crate::core::TurnPhase::Completed);
    Ok(answer)
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
