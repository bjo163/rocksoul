use std::fs;

use anyhow::Result;

use crate::{cli::Command, model::ModelConfig, paths::AppPaths};

const INITIAL_CONFIG: &str = r#"[core]
name = "Rocksoul"
profile = "seed"

[model]
# Configure the model through environment variables:
# ROCKSOUL_BASE_URL, ROCKSOUL_MODEL, ROCKSOUL_API_KEY.
provider = "openai-compatible"

[evolution]
enabled = false
automatic_promotion = false
"#;

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Init => init(),
        Command::Status => status(),
        Command::Doctor => doctor(),
    }
}

pub fn init() -> Result<()> {
    let paths = AppPaths::discover()?;

    for directory in [
        &paths.config_dir,
        &paths.data_dir,
        &paths.cache_dir,
        &paths.workspace_dir,
        &paths.memory_dir,
        &paths.evolution_dir,
        &paths.logs_dir,
    ] {
        fs::create_dir_all(directory)?;
    }

    let config_created = if paths.config_file.exists() {
        false
    } else {
        fs::write(&paths.config_file, INITIAL_CONFIG)?;
        true
    };

    println!("Rocksoul initialized.");
    println!("Config: {}", paths.config_file.display());
    println!("Workspace: {}", paths.workspace_dir.display());
    println!("Memory: {}", paths.memory_dir.display());
    println!("Evolution archive: {}", paths.evolution_dir.display());

    if config_created {
        println!("Created initial config.toml.");
    } else {
        println!("Existing config.toml preserved.");
    }

    Ok(())
}

pub fn status() -> Result<()> {
    let paths = AppPaths::discover()?;
    let model = ModelConfig::from_env();

    println!("Rocksoul v{}", env!("CARGO_PKG_VERSION"));
    println!(
        "Initialization: {}",
        if paths.config_file.is_file() {
            "initialized"
        } else {
            "not initialized (run rocksoul init)"
        }
    );
    println!(
        "Model: {}",
        model
            .as_ref()
            .map(|config| config.model.as_str())
            .unwrap_or("not configured (set ROCKSOUL_MODEL)")
    );
    if model.is_some() {
        println!("Model endpoint: configured (URL withheld from diagnostics)");
    }
    println!("Persistent memory: not implemented in seed release");
    println!("Tool execution: disabled in seed release");
    println!("Self-evolution: disabled in seed release");

    Ok(())
}

pub fn doctor() -> Result<()> {
    let paths = AppPaths::discover()?;

    println!("Rocksoul diagnostics");
    println!("[ok] executable v{}", env!("CARGO_PKG_VERSION"));
    println!("[ok] operating system: {}", std::env::consts::OS);
    println!(
        "[{}] configuration directory: {}",
        if paths.config_dir.is_dir() {
            "ok"
        } else {
            "warn"
        },
        paths.config_dir.display()
    );
    println!(
        "[{}] local data directory: {}",
        if paths.data_dir.is_dir() {
            "ok"
        } else {
            "warn"
        },
        paths.data_dir.display()
    );

    match ModelConfig::from_env() {
        Some(config) => {
            println!("[ok] model configured: {}", config.model);
            println!("[info] model endpoint configured (URL withheld)");
        }
        None => {
            println!("[warn] model not configured; set ROCKSOUL_MODEL to enable chat");
        }
    }

    println!("[info] shell and generated-code execution are disabled.");
    println!("[info] automatic core evolution is disabled.");
    Ok(())
}
