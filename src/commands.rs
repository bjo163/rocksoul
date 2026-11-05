use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};

use crate::{
    cli::{Command, EvalCommand, EvalSuite, MemoryCommand, RidCommand, WorkspaceCommand},
    core::CancellationToken,
    core::{
        TurnEvent, TurnState, VerificationEvidence, VerificationRequest, verify_turn_and_award_xp,
    },
    memory::{MemoryKind, MemoryRecord, MemorySearchFilters, MemoryStore},
    memory_proposal,
    model::ModelConfig,
    paths::AppPaths,
    rid::{RuhIdentity, RuhProfile},
    rid_catalogue::RidCatalogue,
    rid_store,
    task_trace::{self, TaskTraceEvent},
    workspace_artifact::{
        ArtifactOutcome, UserApproval, WorkspaceArtifactProposal, WorkspaceArtifactService,
    },
};

const INITIAL_CONFIG: &str = r#"[core]
name = "Rocksoul"
profile = "seed"

[model]
# Model settings are read from the .env file in the Rocksoul root:
# ROCKSOUL_BASE_URL, ROCKSOUL_MODEL, ROCKSOUL_API_KEY.
provider = "openai-compatible"

[storage]
root = "."

[evolution]
enabled = false
automatic_promotion = false
"#;

const INITIAL_ENV: &str = r#"# Rocksoul model configuration
# This file stays in the Rocksoul installation root beside rocksoul.exe.
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

pub async fn run(command: Command) -> Result<()> {
    match command {
        Command::Workspace { command } => run_workspace(command).await,
        Command::Memory { command } => run_memory(command).await,
        command => run_non_workspace(command),
    }
}

fn run_non_workspace(command: Command) -> Result<()> {
    match command {
        Command::Init => init(),
        Command::Status => status(),
        Command::Doctor => doctor(),
        Command::Tui => crate::tui::run(),
        Command::Rid { command } => run_rid(command),
        Command::Eval { command } => run_eval(command),
        Command::Memory { .. } => unreachable!("memory commands use the async dispatcher"),
        Command::Workspace { .. } => unreachable!("workspace commands use the async dispatcher"),
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
    let (_, memory_created) = MemoryStore::initialize(&paths.memory_dir)?;

    println!("Rocksoul initialized (portable root layout).");
    println!("Root: {}", paths.root_dir.display());
    println!("Config: {}", paths.config_file.display());
    println!("Environment: {}", paths.env_file.display());
    println!("Workspace: {}", paths.workspace_dir.display());
    println!(
        "Memory store: {} ({})",
        paths.memory_dir.join("store.v3.json").display(),
        if memory_created {
            "created"
        } else {
            "preserved"
        }
    );
    println!("Evolution archive: {}", paths.evolution_dir.display());
    println!("Cache: {}", paths.cache_dir.display());
    println!("Logs: {}", paths.logs_dir.display());
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

fn receipt_matches_ignoring_verification_time(existing: &[u8], expected: &[u8]) -> Result<bool> {
    let mut existing: serde_json::Value = serde_json::from_slice(existing)
        .context("existing verification receipt is invalid JSON")?;
    let mut expected: serde_json::Value =
        serde_json::from_slice(expected).context("new verification receipt is invalid JSON")?;
    let timestamps_present = [&existing, &expected].iter().all(|document| {
        document
            .get("verified_at_unix")
            .and_then(serde_json::Value::as_u64)
            .is_some()
    });
    if !timestamps_present {
        return Ok(false);
    }
    for document in [&mut existing, &mut expected] {
        if let Some(object) = document.as_object_mut() {
            object.remove("verified_at_unix");
        }
    }
    Ok(existing == expected)
}

/// Persist a complete, content-free verification receipt beside the task trace.
/// The create-only publish makes retries safe; replay must match every immutable field.
fn persist_workspace_receipt(
    logs_dir: &std::path::Path,
    operation_id: &str,
    receipt: &crate::core::VerificationReceipt,
) -> Result<std::path::PathBuf> {
    use std::io::Write as _;

    let receipt_dir = logs_dir.join("receipts");
    fs::create_dir_all(&receipt_dir).with_context(|| {
        format!(
            "could not create receipt directory {}",
            receipt_dir.display()
        )
    })?;
    let operation_key = operation_id.rsplit(':').next().unwrap_or("unknown");
    let destination = receipt_dir.join(format!("workspace-{operation_key}.json"));
    let document = serde_json::json!({
        "schema_version": 1,
        "operation_id": operation_id,
        "task_id": receipt.task_id(),
        "verification_id": receipt.verification_id(),
        "verified_at_unix": receipt.verified_at_unix(),
        "required_checks_passed": receipt.required_checks_passed(),
        "total_checks": receipt.total_checks(),
        "evidence_refs": receipt.evidence_refs(),
        "claim_sha256": receipt.claim_sha256(),
        "evidence_sha256": receipt.evidence_sha256(),
        "verifier_id": receipt.verifier_id(),
        "verifier_version": receipt.verifier_version(),
        "policy_version": receipt.policy_version(),
        "xp_awarded": receipt.xp_awarded(),
    });
    let mut encoded =
        serde_json::to_vec_pretty(&document).context("could not serialize verification receipt")?;
    encoded.push(b'\n');

    if destination.exists() {
        let existing = fs::read(&destination)
            .with_context(|| format!("could not read receipt {}", destination.display()))?;
        anyhow::ensure!(
            receipt_matches_ignoring_verification_time(&existing, &encoded)?,
            "existing verification receipt conflicts with this operation"
        );
        return Ok(destination);
    }

    let temporary = receipt_dir.join(format!(
        ".workspace-{operation_key}-{}-{}.tmp",
        std::process::id(),
        now_unix()?
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .context("could not create temporary verification receipt")?;
    file.write_all(&encoded)?;
    file.sync_all()?;
    drop(file);

    let publish = fs::hard_link(&temporary, &destination);
    let _ = fs::remove_file(&temporary);
    match publish {
        Ok(()) => Ok(destination),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let existing = fs::read(&destination)
                .with_context(|| format!("could not read receipt {}", destination.display()))?;
            anyhow::ensure!(
                receipt_matches_ignoring_verification_time(&existing, &encoded)?,
                "concurrent verification receipt conflicts with this operation"
            );
            Ok(destination)
        }
        Err(error) => Err(error).context("could not publish verification receipt"),
    }
}

async fn run_workspace(command: WorkspaceCommand) -> Result<()> {
    match command {
        WorkspaceCommand::Trace => {
            let paths = AppPaths::discover()?;
            let trace_path = paths.logs_dir.join("task-trace.v1.jsonl");
            let records = task_trace::read(&trace_path)?;
            if records.is_empty() {
                println!("No task trace records yet.");
            } else {
                for record in records {
                    println!("{}", serde_json::to_string(&record)?);
                }
            }
            let receipt_dir = paths.logs_dir.join("receipts");
            if receipt_dir.is_dir() {
                let mut receipts = fs::read_dir(&receipt_dir)
                    .with_context(|| format!("could not inspect {}", receipt_dir.display()))?
                    .filter_map(std::result::Result::ok)
                    .map(|entry| entry.path())
                    .filter(|path| {
                        path.extension()
                            .is_some_and(|extension| extension == "json")
                    })
                    .collect::<Vec<_>>();
                receipts.sort();
                for receipt in receipts {
                    let content = fs::read_to_string(&receipt)
                        .with_context(|| format!("could not read receipt {}", receipt.display()))?;
                    let parsed: serde_json::Value = serde_json::from_str(&content)
                        .with_context(|| format!("invalid receipt {}", receipt.display()))?;
                    anyhow::ensure!(
                        parsed
                            .get("schema_version")
                            .and_then(serde_json::Value::as_u64)
                            == Some(1),
                        "unsupported verification receipt schema"
                    );
                    println!("receipt: {}", serde_json::to_string(&parsed)?);
                }
            }
        }
        WorkspaceCommand::Write { filename, content } => {
            let proposal = WorkspaceArtifactProposal { filename, content };
            proposal.validate()?;
            let paths = AppPaths::discover()?;
            let operation_id = proposal.operation_id();
            let trace_path = paths.logs_dir.join("task-trace.v1.jsonl");
            task_trace::append(
                &trace_path,
                &operation_id,
                TaskTraceEvent::Proposed,
                &proposal.filename,
                "validated proposal; content omitted",
            )?;
            println!("Proposed workspace artifact: {}", proposal.filename);
            println!(
                "Exact target: {}",
                paths.workspace_dir.join(&proposal.filename).display()
            );
            println!(
                "Content: {} UTF-8 bytes (maximum 16384)",
                proposal.content.len()
            );
            println!("Preview:");
            for line in proposal.content.lines() {
                println!("  {line}");
            }
            println!(
                "To approve this exact artifact, type: {}",
                proposal.approval_phrase()
            );
            print!("Approval: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            use tokio::io::AsyncBufReadExt as _;

            let cancellation = CancellationToken::default();
            let mut confirmation = String::new();
            let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
            tokio::select! {
                result = stdin.read_line(&mut confirmation) => {
                    result.context("could not read workspace approval")?;
                }
                signal = tokio::signal::ctrl_c() => {
                    cancellation.cancel();
                    task_trace::append(
                        &trace_path,
                        &operation_id,
                        TaskTraceEvent::Cancelled,
                        &proposal.filename,
                        "cancelled while awaiting user approval",
                    )?;
                    signal.context("could not receive Ctrl+C during workspace approval")?;
                    anyhow::bail!("workspace approval cancelled");
                }
            }
            let approval = match UserApproval::from_typed_confirmation(&proposal, &confirmation) {
                Ok(approval) => approval,
                Err(error) => {
                    task_trace::append(
                        &trace_path,
                        &operation_id,
                        TaskTraceEvent::Denied,
                        &proposal.filename,
                        "approval phrase rejected",
                    )?;
                    return Err(error);
                }
            };
            task_trace::append(
                &trace_path,
                &operation_id,
                TaskTraceEvent::Approved,
                &proposal.filename,
                "exact typed approval accepted",
            )?;

            let service = WorkspaceArtifactService::new(&paths.workspace_dir);
            let executor_proposal = proposal.clone();
            let executor_approval = approval.clone();
            let executor_cancellation = cancellation.clone();
            let mut execution = tokio::task::spawn_blocking(move || {
                service.execute_approved_with_retry(
                    &executor_proposal,
                    &executor_approval,
                    &executor_cancellation,
                )
            });
            let execution_result = tokio::select! {
                result = &mut execution => result.context("workspace executor task failed")?,
                signal = tokio::signal::ctrl_c() => {
                    cancellation.cancel();
                    signal.context("could not receive Ctrl+C during workspace execution")?;
                    execution.await.context("workspace executor task failed")?
                }
            };
            let outcome = match execution_result {
                Ok(outcome) => outcome,
                Err(error) => {
                    let (event, detail) = if cancellation.is_cancelled() {
                        (
                            TaskTraceEvent::Cancelled,
                            "workspace execution cancelled before publication",
                        )
                    } else {
                        (TaskTraceEvent::Failed, "workspace execution failed")
                    };
                    task_trace::append(
                        &trace_path,
                        &operation_id,
                        event,
                        &proposal.filename,
                        detail,
                    )?;
                    return Err(error);
                }
            };
            task_trace::append(
                &trace_path,
                &operation_id,
                TaskTraceEvent::Executed,
                &proposal.filename,
                "artifact published or identical replay observed",
            )?;
            let (path, bytes, observed_content, outcome_label) = match outcome {
                ArtifactOutcome::Created {
                    path,
                    bytes,
                    observed_content,
                } => (path, bytes, observed_content, "created"),
                ArtifactOutcome::AlreadyPresent {
                    path,
                    bytes,
                    observed_content,
                } => (
                    path,
                    bytes,
                    observed_content,
                    "already present (idempotent replay)",
                ),
            };

            let observed_regular_file = fs::symlink_metadata(&path)
                .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink());
            let timestamp = now_unix()?;
            let mut profile = rid_store::load(&paths.rid_file)
                .context("RID profile is required; run rocksoul init first")?;
            let mut turn = TurnState::new(format!("workspace-{}", proposal.filename), 0);
            turn.apply(TurnEvent::Start)?;
            turn.apply(TurnEvent::PlanAccepted)?;
            turn.apply(TurnEvent::ActionCompleted)?;
            let verification_id = format!("workspace-artifact:{operation_id}");
            let request = VerificationRequest::new(
                operation_id.clone(),
                verification_id,
                timestamp,
                1,
                1,
                vec![
                    VerificationEvidence::new(
                        "artifact-is-regular-file",
                        "true",
                        if observed_regular_file {
                            "true"
                        } else {
                            "false"
                        },
                        format!("workspace-artifact:{operation_id}"),
                        true,
                    ),
                    VerificationEvidence::new(
                        "artifact-readback-matches",
                        proposal.content.as_str(),
                        observed_content.as_str(),
                        format!("workspace-readback:{operation_id}"),
                        true,
                    ),
                ],
            );
            let verification = verify_turn_and_award_xp(&mut turn, &mut profile, request)?;
            rid_store::save(&paths.rid_file, &profile)
                .context("could not persist RID after verification")?;

            if let crate::core::VerificationOutcome::Verified { receipt, xp_record } = &verification
            {
                let receipt_path =
                    persist_workspace_receipt(&paths.logs_dir, &operation_id, receipt)?;
                let newly_awarded = matches!(xp_record, crate::rid::XpRecordOutcome::Recorded);
                let receipt_detail = format!(
                    "receipt={} xp={} xp_outcome={} checks={}/{} evidence={} receipt_path={}",
                    receipt.verification_id(),
                    receipt.xp_awarded(),
                    if newly_awarded {
                        "recorded"
                    } else {
                        "duplicate"
                    },
                    receipt.required_checks_passed(),
                    receipt.total_checks(),
                    receipt.evidence_refs().join(","),
                    receipt_path.display(),
                );
                task_trace::append(
                    &trace_path,
                    &operation_id,
                    TaskTraceEvent::Verified,
                    &proposal.filename,
                    &receipt_detail,
                )?;
            } else {
                task_trace::append(
                    &trace_path,
                    &operation_id,
                    TaskTraceEvent::Failed,
                    &proposal.filename,
                    "independent verification failed; no XP receipt",
                )?;
            }
            println!("Workspace artifact {outcome_label}: {}", path.display());
            println!("Independent read-back verified: {bytes} bytes.");
            println!("RID XP total: {}.", profile.total_xp());
            if let crate::core::VerificationOutcome::Verified { receipt, xp_record } = verification
            {
                let newly_awarded = matches!(xp_record, crate::rid::XpRecordOutcome::Recorded);
                println!(
                    "Verification receipt: {} · {} XP {}",
                    receipt.verification_id(),
                    if newly_awarded {
                        receipt.xp_awarded()
                    } else {
                        0
                    },
                    if newly_awarded {
                        "awarded"
                    } else {
                        "(idempotent replay; no new XP)"
                    }
                );
            }
        }
    }
    Ok(())
}

pub fn status() -> Result<()> {
    let paths = AppPaths::discover()?;
    let model = ModelConfig::from_env();

    println!("Rocksoul v{}", env!("CARGO_PKG_VERSION"));
    println!("Root: {}", paths.root_dir.display());
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

    let memory = MemoryStore::load(&paths.memory_dir)?;
    println!(
        "Memory: {} active records · {} total · {}",
        memory.active_records(now_unix()?).len(),
        memory.records.len(),
        paths.memory_dir.display()
    );
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

    match MemoryStore::load(&paths.memory_dir) {
        Ok(store) => println!(
            "[ok] memory store valid (schema {}, {} records)",
            store.schema_version,
            store.records.len()
        ),
        Err(error) => println!("[error] memory store invalid: {error:#}"),
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

pub fn memory_status() -> Result<()> {
    let paths = AppPaths::discover()?;
    let store = MemoryStore::load(&paths.memory_dir)?;
    print_memory_records(&store.active_records(now_unix()?), "Rocksoul memory");
    println!(
        "Store: {}",
        paths.memory_dir.join("store.v3.json").display()
    );
    Ok(())
}

async fn run_memory(command: MemoryCommand) -> Result<()> {
    let paths = AppPaths::discover()?;
    match command {
        MemoryCommand::Propose {
            kind,
            content,
            tags,
            source_turn_reference,
            provenance,
            expires_in_hours,
        } => {
            let now = now_unix()?;
            let expires_at = expires_in_hours
                .map(|hours| {
                    anyhow::ensure!(hours > 0, "--expires-in-hours must be greater than zero");
                    let seconds = hours
                        .checked_mul(3_600)
                        .context("working-memory expiry is too large")?;
                    now.checked_add(seconds)
                        .context("working-memory expiry timestamp overflow")
                })
                .transpose()?;
            let proposal = memory_proposal::create(
                &paths.memory_dir,
                memory_proposal::MemoryProposalCandidate {
                    kind,
                    content,
                    tags,
                    source_turn_reference,
                    provenance,
                    expires_at_unix: expires_at,
                },
                now,
            )?;
            println!("Memory proposal saved for review; no memory record was written.");
            println!("Proposal: {}", proposal.id);
            println!("Kind: {}", proposal.kind);
            println!("Source turn: {}", proposal.source_turn_reference);
            println!("Provenance: {}", proposal.provenance);
            println!("Fingerprint: {}", proposal.fingerprint);
            println!("Review with: rocksoul memory approve {}", proposal.id);
        }
        MemoryCommand::Proposals => {
            let proposals = memory_proposal::list(&paths.memory_dir)?;
            if proposals.is_empty() {
                println!("No memory proposals yet.");
            }
            for proposal in proposals {
                println!(
                    "{} [{} · {}] source={} fingerprint={}",
                    proposal.id,
                    proposal.kind,
                    proposal.status,
                    proposal.source_turn_reference,
                    proposal.fingerprint
                );
                if let Some(memory_id) = proposal.memory_record_id {
                    println!("  memory record: {memory_id}");
                }
            }
        }
        MemoryCommand::Approve { id } => {
            let proposal = memory_proposal::get_pending(&paths.memory_dir, &id)?;
            println!("Review exact model-proposed memory candidate:");
            println!("Proposal: {}", proposal.id);
            println!("Kind: {}", proposal.kind);
            println!("Source turn: {}", proposal.source_turn_reference);
            println!("Provenance: {}", proposal.provenance);
            println!("Tags: {}", proposal.tags.join(", "));
            println!("Content: {}", proposal.content);
            println!(
                "Expiry: {}",
                proposal
                    .expires_at_unix
                    .map_or_else(|| "none".to_owned(), |value| value.to_string())
            );
            println!(
                "To persist this exact candidate, type: {}",
                proposal.approval_phrase()
            );
            print!("Approval: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            use tokio::io::AsyncBufReadExt as _;
            let mut confirmation = String::new();
            let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
            tokio::select! {
                result = stdin.read_line(&mut confirmation) => {
                    result.context("could not read memory proposal approval")?;
                }
                signal = tokio::signal::ctrl_c() => {
                    memory_proposal::cancel(&paths.memory_dir, &id, now_unix()?)?;
                    signal.context("could not receive Ctrl+C during memory proposal approval")?;
                    anyhow::bail!("memory proposal approval cancelled; no memory was written");
                }
            }
            let memory_id = memory_proposal::approve(
                &paths.memory_dir,
                &id,
                confirmation.trim_end(),
                now_unix()?,
            )?;
            println!("Approved exact proposal and persisted memory record {memory_id}.");
        }
        MemoryCommand::Deny { id } => {
            memory_proposal::deny(&paths.memory_dir, &id, now_unix()?)?;
            println!("Denied memory proposal {id}; no memory record was written.");
        }
        MemoryCommand::CancelProposal { id } => {
            memory_proposal::cancel(&paths.memory_dir, &id, now_unix()?)?;
            println!("Cancelled memory proposal {id}; no memory record was written.");
        }
        MemoryCommand::Export => {
            let store = MemoryStore::load(&paths.memory_dir)?;
            let path = store.export(&paths.memory_dir)?;
            println!("Memory exported to {}", path.display());
        }
        command => {
            MemoryStore::transact(&paths.memory_dir, |store| {
                match command {
                    MemoryCommand::Add {
                        kind,
                        content,
                        tags,
                        expires_in_hours,
                    } => {
                        if expires_in_hours.is_some() && kind != MemoryKind::Working {
                            anyhow::bail!("--expires-in-hours is only valid for working memory");
                        }
                        let now = now_unix()?;
                        let expires_at = expires_in_hours
                            .map(|hours| {
                                if hours == 0 {
                                    anyhow::bail!("--expires-in-hours must be greater than zero");
                                }
                                let seconds = hours
                                    .checked_mul(3_600)
                                    .context("working-memory expiry is too large")?;
                                now.checked_add(seconds)
                                    .context("working-memory expiry timestamp overflow")
                            })
                            .transpose()?;
                        let id = store.add_user_entry(kind, content, tags, expires_at, now)?;
                        println!("Memory saved with explicit user approval.");
                        println!("ID: {id}");
                        println!(
                            "Store: {}",
                            paths.memory_dir.join("store.v3.json").display()
                        );
                    }
                    MemoryCommand::List => {
                        print_memory_records(&store.active_records(now_unix()?), "Rocksoul memory");
                        println!("{} total records", store.records.len());
                    }
                    MemoryCommand::Search {
                        query,
                        kind,
                        source,
                        created_after,
                        created_before,
                        limit,
                    } => {
                        let filters = MemorySearchFilters {
                            kind,
                            source,
                            created_after_unix: created_after,
                            created_before_unix: created_before,
                        };
                        let matches = store.search_ranked(&query, &filters, now_unix()?, limit);
                        println!("Memory search — deterministic lexical ranking");
                        for result in &matches {
                            println!(
                                "- [{}] {} ({}, source={}, created={}, source-confidence={:?})",
                                result.lexical_score,
                                result.record_id,
                                result.kind,
                                result.source,
                                result.created_at_unix,
                                result.source_confidence
                            );
                            println!("  {}", result.content);
                            println!("  {}", result.relevance_explanation);
                        }
                        println!("{} matching records", matches.len());
                    }
                    MemoryCommand::Delete { id } => {
                        if !store.delete(&id) {
                            anyhow::bail!("memory record not found: {id}");
                        }
                        println!("Deleted memory record {id}.");
                    }
                    MemoryCommand::Clear { confirm } => {
                        if !confirm {
                            anyhow::bail!(
                                "this permanently deletes every memory; rerun with --confirm"
                            );
                        }
                        store.clear();
                        println!("All memory records permanently deleted.");
                    }
                    MemoryCommand::PruneExpired { apply } => {
                        let now = now_unix()?;
                        let expired = store.expired_ids(now);
                        if apply {
                            store.prune_expired(now);
                            println!("Forgot {} expired memory record(s).", expired.len());
                        } else {
                            println!(
                                "Dry run: {} expired memory record(s) would be forgotten.",
                                expired.len()
                            );
                            for id in expired {
                                println!("- {id}");
                            }
                            println!(
                                "Run rocksoul memory prune-expired --apply to confirm deletion."
                            );
                        }
                    }
                    MemoryCommand::Export
                    | MemoryCommand::Propose { .. }
                    | MemoryCommand::Proposals
                    | MemoryCommand::Approve { .. }
                    | MemoryCommand::Deny { .. }
                    | MemoryCommand::CancelProposal { .. } => {
                        unreachable!(
                            "proposal and export commands are handled outside the memory transaction"
                        )
                    }
                }
                Ok(())
            })?;
        }
    }
    Ok(())
}

fn print_memory_records(records: &[&MemoryRecord], heading: &str) {
    println!("{heading} — {} records", records.len());
    if records.is_empty() {
        println!("No matching memory records.");
        return;
    }
    for record in records {
        let approval = if record.user_approved {
            "user-approved"
        } else {
            "derived/unapproved"
        };
        let tags = if record.tags.is_empty() {
            String::new()
        } else {
            format!(" · tags: {}", record.tags.join(", "))
        };
        println!(
            "{}  [{} · {}]  {}{}",
            record.id, record.kind, approval, record.content, tags
        );
        println!(
            "    source={} · created={} · expires={}",
            record.source,
            record.created_at_unix,
            record
                .expires_at_unix
                .map_or_else(|| "never".to_owned(), |value| value.to_string())
        );
    }
}

fn run_eval(command: EvalCommand) -> Result<()> {
    match command {
        EvalCommand::List => {
            crate::eval::list();
            Ok(())
        }
        EvalCommand::Run {
            suite: EvalSuite::Core,
        } => {
            let report = crate::eval::run_core()?;
            crate::eval::print_report(&report);
            if report.passed {
                Ok(())
            } else {
                anyhow::bail!("one or more core integrity checks failed")
            }
        }
        EvalCommand::Run {
            suite: EvalSuite::MemoryRetrieval,
        } => {
            let report = crate::eval::run_memory_retrieval()?;
            crate::eval::print_memory_retrieval_report(&report);
            if report.passed {
                Ok(())
            } else {
                anyhow::bail!("memory retrieval evaluation failed its regression gates")
            }
        }
        EvalCommand::Latest => {
            match crate::eval::latest()? {
                Some(report) => crate::eval::print_report(&report),
                None => println!("No evaluation report yet. Run: rocksoul eval run core"),
            }
            Ok(())
        }
        EvalCommand::MemoryLatest => {
            match crate::eval::latest_memory_retrieval()? {
                Some(report) => crate::eval::print_memory_retrieval_report(&report),
                None => println!(
                    "No memory retrieval report yet. Run: rocksoul eval run memory-retrieval"
                ),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        core::{
            TurnEvent, TurnState, VerificationEvidence, VerificationOutcome, VerificationRequest,
            verify_turn_and_award_xp,
        },
        rid::{RuhIdentity, RuhProfile, XpRecordOutcome},
        rid_store,
    };

    use super::{format_age, persist_workspace_receipt};

    #[test]
    fn formats_elapsed_instance_age() {
        assert_eq!(format_age(0), "0s");
        assert_eq!(format_age(65), "1m 5s");
        assert_eq!(format_age(3_661), "1h 1m 1s");
        assert_eq!(format_age(90_061), "1d 1h 1m");
    }

    #[test]
    fn verification_receipt_recovers_after_profile_persist_before_receipt_write() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("rocksoul-receipt-recovery-test-{nonce}"));
        fs::create_dir_all(&root).unwrap();
        let logs = root.join("logs");
        let rid_path = root.join("rid.json");
        let artifact_path = root.join("workspace").join("note.txt");
        fs::create_dir_all(artifact_path.parent().unwrap()).unwrap();
        fs::write(&artifact_path, "expected").unwrap();
        let actual_readback = fs::read_to_string(&artifact_path).unwrap();

        let mut profile = RuhProfile::new(RuhIdentity {
            instance_id: "receipt-recovery-test".to_owned(),
            created_at_unix: 1_700_000_000,
        })
        .unwrap();
        let request = |verified_at_unix| {
            VerificationRequest::new(
                "workspace:note.txt:abc123",
                "workspace-verification:abc123",
                verified_at_unix,
                1,
                1,
                vec![VerificationEvidence::new(
                    "artifact-readback-matches",
                    "expected",
                    actual_readback.as_str(),
                    "workspace-readback:abc123",
                    true,
                )],
            )
        };
        let mut turn = TurnState::new("receipt-task", 0);
        turn.apply(TurnEvent::Start).unwrap();
        turn.apply(TurnEvent::PlanAccepted).unwrap();
        turn.apply(TurnEvent::ActionCompleted).unwrap();
        let first =
            verify_turn_and_award_xp(&mut turn, &mut profile, request(1_700_000_001)).unwrap();
        assert!(matches!(
            first,
            VerificationOutcome::Verified {
                xp_record: XpRecordOutcome::Recorded,
                ..
            }
        ));
        rid_store::save(&rid_path, &profile).unwrap();

        // Simulate a process restart after XP persistence but before receipt persistence.
        drop(profile);
        drop(turn);
        let mut recovered_profile = rid_store::load(&rid_path).unwrap();
        let xp_before_replay = recovered_profile.total_xp();
        let mut replay_turn = TurnState::new("receipt-task-replay", 0);
        replay_turn.apply(TurnEvent::Start).unwrap();
        replay_turn.apply(TurnEvent::PlanAccepted).unwrap();
        replay_turn.apply(TurnEvent::ActionCompleted).unwrap();
        let replay = verify_turn_and_award_xp(
            &mut replay_turn,
            &mut recovered_profile,
            request(1_700_000_002),
        )
        .unwrap();
        let receipt = match replay {
            VerificationOutcome::Verified {
                receipt,
                xp_record: XpRecordOutcome::Duplicate,
            } => receipt,
            other => panic!("expected idempotent replay receipt, got {other:?}"),
        };
        assert_eq!(recovered_profile.total_xp(), xp_before_replay);

        let path = persist_workspace_receipt(&logs, "workspace:note.txt:abc123", &receipt).unwrap();

        // Simulate another restart after receipt publication but before the
        // corresponding Verified trace event. Replay must preserve the first
        // durable receipt even though its observation timestamp is newer.
        let mut second_replay_turn = TurnState::new("receipt-task-replay", 0);
        second_replay_turn.apply(TurnEvent::Start).unwrap();
        second_replay_turn.apply(TurnEvent::PlanAccepted).unwrap();
        second_replay_turn
            .apply(TurnEvent::ActionCompleted)
            .unwrap();
        let second_replay = verify_turn_and_award_xp(
            &mut second_replay_turn,
            &mut recovered_profile,
            request(1_700_000_003),
        )
        .unwrap();
        let second_receipt = match second_replay {
            VerificationOutcome::Verified {
                receipt,
                xp_record: XpRecordOutcome::Duplicate,
            } => receipt,
            other => panic!("expected a second idempotent replay receipt, got {other:?}"),
        };
        assert_eq!(
            persist_workspace_receipt(&logs, "workspace:note.txt:abc123", &second_receipt,)
                .unwrap(),
            path
        );
        assert_eq!(recovered_profile.total_xp(), xp_before_replay);
        let document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(document["schema_version"], 1);
        assert_eq!(document["operation_id"], "workspace:note.txt:abc123");
        assert_eq!(document["verification_id"], "workspace-verification:abc123");
        assert_eq!(document["evidence_refs"][0], "workspace-readback:abc123");
        assert_eq!(
            persist_workspace_receipt(&logs, "workspace:note.txt:abc123", &receipt).unwrap(),
            path
        );

        fs::write(&path, "{}").unwrap();
        assert!(persist_workspace_receipt(&logs, "workspace:note.txt:abc123", &receipt).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
