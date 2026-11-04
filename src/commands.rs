use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};

use crate::{
    cli::{Command, RidCommand},
    model::ModelConfig,
    paths::AppPaths,
    rid::{RuhIdentity, RuhProfile},
    rid_catalogue::RidCatalogue,
    rid_store,
};

const INITIAL_CONFIG: &str = r#"[core]
name = "Rocksoul"
profile = "seed"

[model]
# Model settings are read from the dotenv file beside rocksoul.exe:
# ROCKSOUL_BASE_URL, ROCKSOUL_MODEL, ROCKSOUL_API_KEY.
provider = "openai-compatible"

[evolution]
enabled = false
automatic_promotion = false
"#;

const INITIAL_ENV: &str = r#"# Rocksoul model configuration
# This file must stay beside rocksoul.exe.
# Existing Windows process environment variables take precedence.

# Local Ollama example: uncomment and set the model you installed.
ROCKSOUL_BASE_URL=http://localhost:11434/v1
ROCKSOUL_MODEL=
ROCKSOUL_API_KEY=

# Hosted OpenAI-compatible endpoint example:
# ROCKSOUL_BASE_URL=https://provider.example/v1
# ROCKSOUL_MODEL=your-model-id
# ROCKSOUL_API_KEY=your-api-key
"#;

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Init => init(),
        Command::Status => status(),
        Command::Doctor => doctor(),
        Command::Rid { command } => run_rid(command),
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
    fs::create_dir_all(paths.rid_file.parent().context("RID path has no parent")?)?;

    let config_created = if paths.config_file.exists() {
        false
    } else {
        fs::write(&paths.config_file, INITIAL_CONFIG)?;
        true
    };

    let env_created = if paths.env_file.exists() {
        false
    } else {
        fs::write(&paths.env_file, INITIAL_ENV)?;
        true
    };

    let created_at_unix = now_unix()?;
    let identity = RuhIdentity {
        instance_id: format!("rocksoul-{created_at_unix}-{}", std::process::id()),
        created_at_unix,
    };
    let (_, rid_created) = rid_store::initialize(&paths.rid_file, identity)?;

    println!("Rocksoul initialized.");
    println!("Config: {}", paths.config_file.display());
    println!("Environment: {}", paths.env_file.display());
    println!("Workspace: {}", paths.workspace_dir.display());
    println!("Memory: {}", paths.memory_dir.display());
    println!("Evolution archive: {}", paths.evolution_dir.display());
    println!(
        "RID profile: {} ({})",
        paths.rid_file.display(),
        if rid_created { "created" } else { "preserved" }
    );

    if config_created {
        println!("Created initial config.toml.");
    } else {
        println!("Existing config.toml preserved.");
    }

    if env_created {
        println!("Created .env beside rocksoul.exe. Edit it and restart Rocksoul.");
    } else {
        println!("Existing .env preserved.");
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
        "Environment file: {}",
        if paths.env_file.is_file() {
            "found beside executable"
        } else {
            "not found (run rocksoul init)"
        }
    );
    println!(
        "Model: {}",
        model
            .as_ref()
            .map(|config| config.model.as_str())
            .unwrap_or("not configured (edit .env beside rocksoul.exe)")
    );
    if model.is_some() {
        println!("Model endpoint: configured (URL withheld from diagnostics)");
    }

    if paths.rid_file.is_file() {
        let profile = rid_store::load(&paths.rid_file)?;
        println!("RID: level {} · {} XP", profile.level(), profile.total_xp());
    } else {
        println!("RID: not initialized (run rocksoul init)");
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
    println!(
        "[{}] .env beside executable",
        if paths.env_file.is_file() {
            "ok"
        } else {
            "warn"
        }
    );

    if paths.rid_file.is_file() {
        match rid_store::load(&paths.rid_file) {
            Ok(profile) => println!(
                "[ok] RID profile valid (level {}, {} XP, {} stats)",
                profile.level(),
                profile.total_xp(),
                profile.stats().len()
            ),
            Err(error) => println!("[error] RID profile invalid: {error:#}"),
        }
    } else {
        println!("[warn] RID profile missing; run rocksoul init");
    }

    match ModelConfig::from_env() {
        Some(config) => {
            println!("[ok] model configured: {}", config.model);
            println!("[info] model endpoint configured (URL withheld)");
        }
        None => {
            println!("[warn] model not configured; edit .env beside rocksoul.exe");
        }
    }

    println!("[info] shell and generated-code execution are disabled.");
    println!("[info] automatic core evolution is disabled.");
    Ok(())
}

pub fn rid_status() -> Result<()> {
    let (paths, profile) = load_rid_profile()?;
    let age = format_age(profile.age_seconds_at(now_unix()?));
    let rated = profile
        .stats()
        .iter()
        .filter(|stat| stat.score.is_some())
        .count();

    println!("Ruh Information (RID)");
    println!("Instance: {}", profile.identity().instance_id);
    println!("Age: {age}");
    println!("Level: {}", profile.level());
    println!("XP: {}", profile.total_xp());
    println!("Stats rated: {rated}/99");
    println!("Profile: {}", paths.rid_file.display());
    println!("Naming layer: Asmaul Husna-inspired technical dimensions; not divine attributes.");

    Ok(())
}

fn run_rid(command: RidCommand) -> Result<()> {
    match command {
        RidCommand::Status => rid_status(),
        RidCommand::Stats => rid_stats(),
        RidCommand::History => rid_history(),
    }
}

fn rid_stats() -> Result<()> {
    let (_, profile) = load_rid_profile()?;
    let catalogue = RidCatalogue::embedded()?;

    println!(
        "RID capability stats — {} slots · catalogue {}",
        profile.stats().len(),
        catalogue.catalogue_version
    );
    println!("Name labels are inspirational only; technical scores measure software capability.");

    for stat in profile.stats() {
        let entry = catalogue
            .by_slot(stat.id.value())
            .context("RID profile references a slot missing from the embedded catalogue")?;
        let score = stat
            .score
            .map(|value| format!("{value}/100"))
            .unwrap_or_else(|| "unrated".to_owned());
        let confidence = stat
            .confidence
            .map(|value| format!("{value}%"))
            .unwrap_or_else(|| "—".to_owned());

        println!(
            "{:02}  {:<20}  {:<30}  {}  (confidence: {})",
            stat.id.value(),
            entry.name.transliteration,
            entry.technical_stat.label,
            score,
            confidence
        );
        println!("    {} · {}", entry.name.arabic, entry.technical_stat.key);
    }

    Ok(())
}

fn rid_history() -> Result<()> {
    let (_, profile) = load_rid_profile()?;
    let events = profile.xp_events();

    println!("RID progression history — {} events", events.len());
    if events.is_empty() {
        println!("No XP events recorded yet.");
        return Ok(());
    }

    for event in events {
        println!(
            "{}  +{} XP  {:?}  event={}  evidence={}",
            event.occurred_at_unix, event.amount, event.source, event.event_id, event.evidence_ref
        );
    }
    println!(
        "Total: {} XP · Level {}",
        profile.total_xp(),
        profile.level()
    );
    Ok(())
}

fn load_rid_profile() -> Result<(AppPaths, RuhProfile)> {
    let paths = AppPaths::discover()?;
    let profile = rid_store::load(&paths.rid_file).with_context(|| {
        format!(
            "RID is unavailable; run rocksoul init first (profile: {})",
            paths.rid_file.display()
        )
    })?;
    Ok((paths, profile))
}

fn now_unix() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_secs())
}

fn format_age(seconds: u64) -> String {
    let years = seconds / 31_557_600;
    let days = (seconds % 31_557_600) / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let remaining = seconds % 60;

    if years > 0 {
        format!("{years}y {days}d {hours}h")
    } else if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {remaining}s")
    } else if minutes > 0 {
        format!("{minutes}m {remaining}s")
    } else {
        format!("{remaining}s")
    }
}

#[cfg(test)]
mod tests {
    use super::format_age;

    #[test]
    fn formats_elapsed_instance_age() {
        assert_eq!(format_age(0), "0s");
        assert_eq!(format_age(65), "1m 5s");
        assert_eq!(format_age(3_661), "1h 1m 1s");
        assert_eq!(format_age(90_061), "1d 1h 1m");
    }
}
