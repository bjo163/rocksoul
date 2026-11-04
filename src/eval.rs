use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{
    core::{TurnEvent, TurnPhase, TurnState},
    paths::AppPaths,
    rid::{RuhIdentity, RuhProfile, level_for_xp},
    rid_catalogue::RidCatalogue,
};

pub const CORE_SUITE_ID: &str = "rocksoul.core-integrity";
pub const CORE_SUITE_VERSION: &str = "1.0.0";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EvaluationCheck {
    pub id: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EvaluationReport {
    pub schema_version: u16,
    pub suite_id: String,
    pub suite_version: String,
    pub rocksoul_version: String,
    pub platform: String,
    pub architecture: String,
    pub evaluated_at_unix: u64,
    pub passed: bool,
    pub checks: Vec<EvaluationCheck>,
}

impl EvaluationReport {
    pub fn passed_count(&self) -> usize {
        self.checks.iter().filter(|check| check.passed).count()
    }

    pub fn failed_count(&self) -> usize {
        self.checks.len().saturating_sub(self.passed_count())
    }
}

pub fn list() {
    println!("{CORE_SUITE_ID}  v{CORE_SUITE_VERSION}  offline / deterministic");
    println!("  Validates lifecycle, retry bounds, terminal immutability, RID and catalogue invariants.");
    println!("  External benchmark adapters: planned; no external datasets are downloaded.");
}

pub fn run_core() -> Result<EvaluationReport> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_secs();
    let mut checks = Vec::new();

    let mut turn = TurnState::new("eval-lifecycle", 1);
    let lifecycle = turn.apply(TurnEvent::Start).is_ok()
        && turn.phase == TurnPhase::Planning
        && turn.apply(TurnEvent::PlanAccepted).is_ok()
        && turn.phase == TurnPhase::Executing
        && turn.apply(TurnEvent::ActionCompleted).is_ok()
        && turn.phase == TurnPhase::Verifying
        && turn.apply(TurnEvent::VerificationPassed).is_ok()
        && turn.phase == TurnPhase::Completed;
    checks.push(check("kernel.lifecycle", lifecycle, "Idle → Planning → Executing → Verifying → Completed"));

    let mut retry = TurnState::new("eval-retry", 1);
    let retry_ok = retry.apply(TurnEvent::Start).is_ok()
        && retry.apply(TurnEvent::PlanAccepted).is_ok()
        && retry.apply(TurnEvent::ActionCompleted).is_ok()
        && retry.apply(TurnEvent::RetryRequested).is_ok()
        && retry.retry_count == 1
        && retry.apply(TurnEvent::PlanAccepted).is_ok()
        && retry.apply(TurnEvent::ActionCompleted).is_ok()
        && retry.apply(TurnEvent::RetryRequested).is_err()
        && retry.retry_count == 1;
    checks.push(check("kernel.retry_budget", retry_ok, "Retry limit is enforced without incrementing past the configured maximum"));

    let mut terminal = TurnState::new("eval-terminal", 0);
    let terminal_ok = terminal.apply(TurnEvent::Start).is_ok()
        && terminal.apply(TurnEvent::Cancel).is_ok()
        && terminal.phase == TurnPhase::Cancelled
        && terminal.apply(TurnEvent::Start).is_err()
        && terminal.phase == TurnPhase::Cancelled;
    checks.push(check("kernel.terminal_immutable", terminal_ok, "Cancelled turns reject further transitions"));

    let profile = RuhProfile::new(RuhIdentity {
        instance_id: "rocksoul-evaluation".to_owned(),
        created_at_unix: now.saturating_sub(60),
    });
    let rid_ok = match profile {
        Ok(profile) => profile.validate().is_ok()
            && profile.stats().len() == 99
            && profile.stats().iter().all(|stat| stat.score.is_none())
            && profile.age_seconds_at(now) == 60,
        Err(_) => false,
    };
    checks.push(check("rid.identity_and_99_unrated", rid_ok, "RID initializes 99 unrated slots and derives age from creation time"));

    let catalogue_ok = RidCatalogue::embedded()
        .and_then(|catalogue| {
            catalogue.validate()?;
            anyhow::ensure!(catalogue.entries.len() == 99, "expected 99 catalogue entries");
            let unique_keys = catalogue.entries.iter()
                .map(|entry| entry.technical_stat.key.as_str())
                .collect::<std::collections::HashSet<_>>();
            anyhow::ensure!(unique_keys.len() == 99, "technical stat keys are not unique");
            Ok(())
        })
        .is_ok();
    checks.push(check("rid.catalogue_integrity", catalogue_ok, "Embedded catalogue contains 99 valid entries and unique technical keys"));

    let level_ok = level_for_xp(0) == 1
        && level_for_xp(99) == 1
        && level_for_xp(100) == 2
        && level_for_xp(299) == 2
        && level_for_xp(300) == 3
        && level_for_xp(600) == 4;
    checks.push(check("rid.level_curve_v1", level_ok, "Level thresholds match the deterministic v1 curve"));

    let report = EvaluationReport {
        schema_version: 1,
        suite_id: CORE_SUITE_ID.to_owned(),
        suite_version: CORE_SUITE_VERSION.to_owned(),
        rocksoul_version: env!("CARGO_PKG_VERSION").to_owned(),
        platform: std::env::consts::OS.to_owned(),
        architecture: std::env::consts::ARCH.to_owned(),
        evaluated_at_unix: now,
        passed: checks.iter().all(|item| item.passed),
        checks,
    };

    persist_report(&report)?;
    Ok(report)
}

pub fn latest() -> Result<Option<EvaluationReport>> {
    let path = latest_report_path()?;
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(&path).with_context(|| format!("could not read {}", path.display()))?;
    let report = serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid evaluation report at {}", path.display()))?;
    Ok(Some(report))
}

fn persist_report(report: &EvaluationReport) -> Result<()> {
    let path = latest_report_path()?;
    let parent = path.parent().context("evaluation report path has no parent")?;
    fs::create_dir_all(parent).with_context(|| format!("could not create {}", parent.display()))?;
    let bytes = serde_json::to_vec_pretty(report)?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, bytes).with_context(|| format!("could not write {}", temporary.display()))?;
    fs::rename(&temporary, &path).with_context(|| format!("could not replace {}", path.display()))?;
    Ok(())
}

fn latest_report_path() -> Result<PathBuf> {
    Ok(AppPaths::discover()?.data_dir.join("evaluations").join("core").join("latest.json"))
}

fn check(id: &str, passed: bool, detail: &str) -> EvaluationCheck {
    EvaluationCheck { id: id.to_owned(), passed, detail: detail.to_owned() }
}

pub fn print_report(report: &EvaluationReport) {
    println!("Rocksoul evaluation — {} v{}", report.suite_id, report.suite_version);
    println!("Runtime: {} / {} · Rocksoul {}", report.platform, report.architecture, report.rocksoul_version);
    println!("Result: {} ({}/{} passed)", if report.passed { "PASS" } else { "FAIL" }, report.passed_count(), report.checks.len());
    for item in &report.checks {
        println!("  [{}] {} — {}", if item.passed { "PASS" } else { "FAIL" }, item.id, item.detail);
    }
}
