use std::io::{self, BufRead, Write};
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;

use crate::{
    commands,
    model::{ChatMessage, ModelConfig},
};

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

                match model.complete(&client, &messages).await {
                    Ok(answer) => {
                        println!("\n{answer}\n");
                        messages.push(ChatMessage::new("assistant", answer));
                        trim_history(&mut messages);
                    }
                    Err(error) => {
                        messages.pop();
                        eprintln!("Model request failed: {error:#}");
                    }
                }
            }
        }
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
