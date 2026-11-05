use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        CancellationToken, TurnEvent, TurnPhase, TurnState, VerificationEvidence,
        VerificationOutcome, VerificationRequest, verify_turn_and_award_xp,
    },
    memory::{
        MEMORY_SCHEMA_VERSION, MemoryKind, MemoryRecord, MemorySearchFilters, MemoryStore,
    },
    paths::AppPaths,
    rid::{RuhIdentity, RuhProfile, XpRecordOutcome, level_for_xp},
    rid_catalogue::RidCatalogue,
    task_trace::{self, TaskTraceEvent},
    workspace_artifact::{
        ArtifactOutcome, UserApproval, WorkspaceArtifactProposal, WorkspaceArtifactService,
    },
};

pub const CORE_SUITE_ID: &str = "rocksoul.core-integrity";
pub const CORE_SUITE_VERSION: &str = "1.2.0";

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
    println!(
        "  Validates lifecycle, retry bounds, verified XP, workspace approval/read-back mismatch/replay, retry/cancellation boundaries, task trace, RID and catalogue invariants."
    );
    println!("rocksoul.memory-retrieval  v1.0.0  offline / deterministic");
    println!("  Measures lexical precision/recall, stale ranking, expiry leakage, and forgetting leakage.");
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
    checks.push(check(
        "kernel.lifecycle",
        lifecycle,
        "Idle → Planning → Executing → Verifying → Completed",
    ));

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
    checks.push(check(
        "kernel.retry_budget",
        retry_ok,
        "Retry limit is enforced without incrementing past the configured maximum",
    ));

    let mut terminal = TurnState::new("eval-terminal", 0);
    let terminal_ok = terminal.apply(TurnEvent::Start).is_ok()
        && terminal.apply(TurnEvent::Cancel).is_ok()
        && terminal.phase == TurnPhase::Cancelled
        && terminal.apply(TurnEvent::Start).is_err()
        && terminal.phase == TurnPhase::Cancelled;
    checks.push(check(
        "kernel.terminal_immutable",
        terminal_ok,
        "Cancelled turns reject further transitions",
    ));

    let mut verified_turn = TurnState::new("eval-verified", 0);
    let mut verified_profile = RuhProfile::new(RuhIdentity {
        instance_id: "rocksoul-eval-verifier".to_owned(),
        created_at_unix: now.max(1),
    })?;
    verified_turn.apply(TurnEvent::Start)?;
    verified_turn.apply(TurnEvent::PlanAccepted)?;
    verified_turn.apply(TurnEvent::ActionCompleted)?;
    let verified_request = || {
        VerificationRequest::new(
            "eval-task",
            "eval-verification-idempotency",
            now.max(1),
            1,
            1,
            vec![VerificationEvidence::new(
                "expected-output",
                "observed",
                "observed",
                "eval://core/verified-output",
                true,
            )],
        )
    };
    let first_verification = verify_turn_and_award_xp(
        &mut verified_turn,
        &mut verified_profile,
        verified_request(),
    );
    let first_ok = matches!(
        first_verification,
        Ok(VerificationOutcome::Verified {
            xp_record: XpRecordOutcome::Recorded,
            ..
        })
    ) && verified_profile.total_xp() > 0
        && verified_turn.phase == TurnPhase::Completed;
    checks.push(check(
        "kernel.verified_xp_gate",
        first_ok,
        "XP is awarded only after a required evidence check passes",
    ));

    let mut replay_turn = TurnState::new("eval-replay", 0);
    replay_turn.apply(TurnEvent::Start)?;
    replay_turn.apply(TurnEvent::PlanAccepted)?;
    replay_turn.apply(TurnEvent::ActionCompleted)?;
    let xp_before_replay = verified_profile.total_xp();
    let replay =
        verify_turn_and_award_xp(&mut replay_turn, &mut verified_profile, verified_request());
    let replay_ok = matches!(
        replay,
        Ok(VerificationOutcome::Verified {
            xp_record: XpRecordOutcome::Duplicate,
            ..
        })
    ) && verified_profile.total_xp() == xp_before_replay;
    checks.push(check(
        "kernel.xp_replay_idempotency",
        replay_ok,
        "Replaying the same verified event does not duplicate XP",
    ));

    let mut rejected_turn = TurnState::new("eval-rejected", 0);
    let mut rejected_profile = RuhProfile::new(RuhIdentity {
        instance_id: "rocksoul-eval-rejected".to_owned(),
        created_at_unix: now.max(1),
    })?;
    rejected_turn.apply(TurnEvent::Start)?;
    rejected_turn.apply(TurnEvent::PlanAccepted)?;
    rejected_turn.apply(TurnEvent::ActionCompleted)?;
    let rejected_request = VerificationRequest::new(
        "eval-task-rejected",
        "eval-verification-rejected",
        now.max(1),
        1,
        1,
        vec![VerificationEvidence::new(
            "expected-output",
            "expected",
            "different",
            "eval://core/rejected-output",
            true,
        )],
    );
    let rejected =
        verify_turn_and_award_xp(&mut rejected_turn, &mut rejected_profile, rejected_request);
    let rejection_ok = matches!(rejected, Ok(VerificationOutcome::Rejected { .. }))
        && rejected_profile.total_xp() == 0
        && rejected_turn.phase == TurnPhase::Failed;
    checks.push(check(
        "kernel.rejected_verification_no_xp",
        rejection_ok,
        "A failed required evidence check awards no XP",
    ));

    let profile = RuhProfile::new(RuhIdentity {
        instance_id: "rocksoul-evaluation".to_owned(),
        created_at_unix: now.saturating_sub(60),
    });
    let rid_ok = match profile {
        Ok(profile) => {
            profile.validate().is_ok()
                && profile.stats().len() == 99
                && profile.stats().iter().all(|stat| stat.score.is_none())
                && profile.age_seconds_at(now) == 60
        }
        Err(_) => false,
    };
    checks.push(check(
        "rid.identity_and_99_unrated",
        rid_ok,
        "RID initializes 99 unrated slots and derives age from creation time",
    ));

    let catalogue_ok = RidCatalogue::embedded()
        .and_then(|catalogue| {
            catalogue.validate()?;
            anyhow::ensure!(
                catalogue.entries.len() == 99,
                "expected 99 catalogue entries"
            );
            let unique_keys = catalogue
                .entries
                .iter()
                .map(|entry| entry.technical_stat.key.as_str())
                .collect::<std::collections::HashSet<_>>();
            anyhow::ensure!(
                unique_keys.len() == 99,
                "technical stat keys are not unique"
            );
            Ok(())
        })
        .is_ok();
    checks.push(check(
        "rid.catalogue_integrity",
        catalogue_ok,
        "Embedded catalogue contains 99 valid entries and unique technical keys",
    ));

    let level_ok = level_for_xp(0) == 1
        && level_for_xp(99) == 1
        && level_for_xp(100) == 2
        && level_for_xp(299) == 2
        && level_for_xp(300) == 3
        && level_for_xp(600) == 4;
    checks.push(check(
        "rid.level_curve_v1",
        level_ok,
        "Level thresholds match the deterministic v1 curve",
    ));

    let memory = MemoryStore::default();
    let memory_ok = memory.schema_version == MEMORY_SCHEMA_VERSION
        && memory.validate().is_ok()
        && memory.records.is_empty();
    checks.push(check(
        "memory.versioned_empty_default",
        memory_ok,
        "Memory starts empty with a supported schema and no implicit conversation ingestion",
    ));

    let explicit_memory_ok = {
        let mut store = MemoryStore::default();
        store
            .add_user_entry(
                MemoryKind::Semantic,
                "explicit test memory".to_owned(),
                vec!["evaluation".to_owned()],
                None,
                now.max(1),
            )
            .is_ok()
            && store.records.len() == 1
            && store.records[0].user_approved
            && store.records[0].source == "user_explicit"
    };
    checks.push(check(
        "memory.explicit_approval_gate",
        explicit_memory_ok,
        "Memory entries are created through explicit user-approved writes",
    ));

    stage1_workspace_checks(now, &mut checks)?;

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

fn stage1_workspace_checks(now: u64, checks: &mut Vec<EvaluationCheck>) -> Result<()> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "rocksoul-stage1-eval-{now}-{nonce}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    let item = WorkspaceArtifactProposal {
        filename: "stage1-eval.txt".to_owned(),
        content: "approved and independently observed".to_owned(),
    };
    let invalid = WorkspaceArtifactProposal {
        filename: "../escape.txt".to_owned(),
        content: "must be rejected".to_owned(),
    };
    let approval_gate = invalid.validate().is_err()
        && UserApproval::from_typed_confirmation(&item, "WRITE different.txt").is_err();
    checks.push(check(
        "stage1.proposal_and_approval_boundary",
        approval_gate,
        "Traversal and mismatched typed approval are rejected before execution",
    ));

    let approval = UserApproval::from_typed_confirmation(&item, &item.approval_phrase());
    let service = WorkspaceArtifactService::new(root.join("workspace"));
    let executed = approval.as_ref().ok().and_then(|approved| {
        service
            .execute_approved_with_retry(&item, approved, &CancellationToken::default())
            .ok()
    });
    let observed = match executed {
        Some(ArtifactOutcome::Created {
            path,
            observed_content,
            ..
        }) => Some((path, observed_content)),
        _ => None,
    };
    let readback_ok = observed.as_ref().is_some_and(|(path, content)| {
        fs::symlink_metadata(path)
            .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
            && content == &item.content
            && fs::read(path).is_ok_and(|bytes| bytes == item.content.as_bytes())
    });
    checks.push(check(
        "stage1.approved_create_and_readback",
        readback_ok,
        "Approved create-only execution reopens the actual artifact and matches observed bytes",
    ));

    let mut profile = RuhProfile::new(RuhIdentity {
        instance_id: format!("rocksoul-stage1-eval-{now}"),
        created_at_unix: now.max(1),
    })?;
    let operation_id = item.operation_id();
    let verification_id = format!("stage1-eval:{operation_id}");
    let verification_ok = if let Some((path, content)) = &observed {
        let mut turn = TurnState::new("stage1-eval-task", 0);
        turn.apply(TurnEvent::Start)?;
        turn.apply(TurnEvent::PlanAccepted)?;
        turn.apply(TurnEvent::ActionCompleted)?;
        let request = VerificationRequest::new(
            operation_id.clone(),
            verification_id.clone(),
            now.max(1),
            1,
            1,
            vec![
                VerificationEvidence::new(
                    "artifact-is-regular-file",
                    "true",
                    if fs::symlink_metadata(path).is_ok_and(|metadata| {
                        metadata.is_file() && !metadata.file_type().is_symlink()
                    }) {
                        "true"
                    } else {
                        "false"
                    },
                    format!("workspace-artifact:{operation_id}"),
                    true,
                ),
                VerificationEvidence::new(
                    "artifact-readback-matches",
                    item.content.as_str(),
                    content.as_str(),
                    format!("workspace-readback:{operation_id}"),
                    true,
                ),
            ],
        );
        matches!(
            verify_turn_and_award_xp(&mut turn, &mut profile, request),
            Ok(VerificationOutcome::Verified {
                xp_record: XpRecordOutcome::Recorded,
                ..
            })
        )
    } else {
        false
    };
    checks.push(check(
        "stage1.observed_evidence_awards_xp",
        verification_ok && profile.total_xp() > 0,
        "Only trusted file/read-back evidence passes verification and records XP",
    ));

    let mismatch_item = WorkspaceArtifactProposal {
        filename: "stage1-mismatch.txt".to_owned(),
        content: "approved bytes".to_owned(),
    };
    let mismatch_result =
        UserApproval::from_typed_confirmation(&mismatch_item, &mismatch_item.approval_phrase())
            .and_then(|approved| {
                service.execute_approved_with_retry(
                    &mismatch_item,
                    &approved,
                    &CancellationToken::default(),
                )
            });
    let mismatch_no_xp = if let Ok(ArtifactOutcome::Created { path, .. }) = mismatch_result {
        fs::write(&path, "tampered bytes")?;
        let actual_readback = fs::read_to_string(&path)?;
        let mut mismatch_profile = RuhProfile::new(RuhIdentity {
            instance_id: format!("rocksoul-stage1-mismatch-{now}"),
            created_at_unix: now.max(1),
        })?;
        let mut mismatch_turn = TurnState::new("stage1-mismatch-task", 0);
        mismatch_turn.apply(TurnEvent::Start)?;
        mismatch_turn.apply(TurnEvent::PlanAccepted)?;
        mismatch_turn.apply(TurnEvent::ActionCompleted)?;
        let mismatch_request = VerificationRequest::new(
            format!("{}:mismatch", mismatch_item.operation_id()),
            format!("stage1-mismatch:{}", mismatch_item.operation_id()),
            now.max(1),
            1,
            1,
            vec![VerificationEvidence::new(
                "artifact-readback-matches",
                mismatch_item.content.as_str(),
                actual_readback.as_str(),
                format!(
                    "workspace-readback-mismatch:{}",
                    mismatch_item.operation_id()
                ),
                true,
            )],
        );
        matches!(
            verify_turn_and_award_xp(&mut mismatch_turn, &mut mismatch_profile, mismatch_request,),
            Ok(VerificationOutcome::Rejected { .. })
        ) && mismatch_profile.total_xp() == 0
    } else {
        false
    };
    checks.push(check(
        "stage1.readback_mismatch_no_xp",
        mismatch_no_xp,
        "A real post-write byte mismatch is rejected and awards no XP",
    ));

    let replay_ok = verification_ok
        && if let Ok(approved) = approval.as_ref() {
            matches!(
                service.execute_approved_with_retry(&item, approved, &CancellationToken::default()),
                Ok(ArtifactOutcome::AlreadyPresent { .. })
            )
        } else {
            false
        };
    let xp_before_replay = profile.total_xp();
    let replay_verification = if replay_ok {
        observed.as_ref().and_then(|(path, _)| {
            fs::read_to_string(path).ok().map(|readback| {
                let mut replay_turn = TurnState::new("stage1-eval-replay", 0);
                let _ = replay_turn.apply(TurnEvent::Start);
                let _ = replay_turn.apply(TurnEvent::PlanAccepted);
                let _ = replay_turn.apply(TurnEvent::ActionCompleted);
                let request = VerificationRequest::new(
                    operation_id.clone(),
                    verification_id.clone(),
                    now.max(1),
                    1,
                    1,
                    vec![
                        VerificationEvidence::new(
                            "artifact-is-regular-file",
                            "true",
                            if fs::symlink_metadata(path).is_ok_and(|metadata| {
                                metadata.is_file() && !metadata.file_type().is_symlink()
                            }) {
                                "true"
                            } else {
                                "false"
                            },
                            format!("workspace-artifact:{operation_id}"),
                            true,
                        ),
                        VerificationEvidence::new(
                            "artifact-readback-matches",
                            item.content.as_str(),
                            readback.as_str(),
                            format!("workspace-readback:{}", item.filename),
                            true,
                        ),
                    ],
                );
                verify_turn_and_award_xp(&mut replay_turn, &mut profile, request)
            })
        })
    } else {
        None
    };
    let replay_xp_ok = matches!(
        replay_verification,
        Some(Ok(VerificationOutcome::Verified {
            xp_record: XpRecordOutcome::Duplicate,
            ..
        }))
    ) && profile.total_xp() == xp_before_replay;
    checks.push(check(
        "stage1.replay_is_idempotent",
        replay_ok && replay_xp_ok,
        "Identical artifact replay and verification do not add a second XP event",
    ));

    let cancelled_item = WorkspaceArtifactProposal {
        filename: "cancelled.txt".to_owned(),
        content: "must not be published".to_owned(),
    };
    let cancellation = CancellationToken::default();
    cancellation.cancel();
    let cancelled = service
        .execute_approved(
            &cancelled_item,
            &UserApproval::from_typed_confirmation(
                &cancelled_item,
                &cancelled_item.approval_phrase(),
            )?,
            &cancellation,
        )
        .is_err()
        && !root.join("workspace").join("cancelled.txt").exists();
    checks.push(check(
        "stage1.pre_cancel_blocks_publication",
        cancelled,
        "A cancelled operation cannot publish an artifact",
    ));

    fs::create_dir_all(root.join("logs"))?;
    let trace_path = root.join("logs/task-trace.v1.jsonl");
    task_trace::append(
        &trace_path,
        &operation_id,
        TaskTraceEvent::Proposed,
        &item.filename,
        "offline stage 1 evaluation",
    )?;
    let trace_ok = task_trace::read(&trace_path).is_ok_and(|records| records.len() == 1);
    checks.push(check(
        "stage1.trace_schema_roundtrip",
        trace_ok,
        "Versioned JSONL task trace can be appended and read back",
    ));

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let outside = root.join("outside.txt");
        fs::write(&outside, "preserve")?;
        let link_root = root.join("linked-workspace");
        fs::create_dir_all(&link_root)?;
        symlink(&outside, link_root.join("linked.txt"))?;
        let link_item = WorkspaceArtifactProposal {
            filename: "linked.txt".to_owned(),
            content: "must not follow symlink".to_owned(),
        };
        let symlink_rejected = WorkspaceArtifactService::new(&link_root)
            .execute_approved(
                &link_item,
                &UserApproval::from_typed_confirmation(&link_item, &link_item.approval_phrase())?,
                &CancellationToken::default(),
            )
            .is_err()
            && fs::read_to_string(&outside).is_ok_and(|value| value == "preserve");
        checks.push(check(
            "stage1.target_symlink_rejected",
            symlink_rejected,
            "A symlink target is rejected without writing through it",
        ));
    }

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemoryRetrievalCaseReport {
    pub case_id: String,
    pub expected_ids: Vec<String>,
    pub retrieved_ids: Vec<String>,
    pub precision_at_k: f64,
    pub recall_at_k: f64,
    pub stale_results_at_k: usize,
    pub expired_leakage: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemoryRetrievalEvaluationReport {
    pub schema_version: u16,
    pub suite_id: String,
    pub suite_version: String,
    pub evaluated_at_unix: u64,
    pub top_k: usize,
    pub precision_at_k: f64,
    pub recall_at_k: f64,
    pub stale_results_at_k: usize,
    pub expired_leakage: usize,
    pub deletion_leakage: usize,
    pub passed: bool,
    pub cases: Vec<MemoryRetrievalCaseReport>,
}

#[derive(Deserialize)]
struct MemoryRetrievalFixture {
    schema_version: u16,
    suite_id: String,
    suite_version: String,
    top_k: usize,
    cases: Vec<MemoryRetrievalFixtureCase>,
}

#[derive(Deserialize)]
struct MemoryRetrievalFixtureCase {
    id: String,
    query: String,
    now_unix: u64,
    expected_ids: Vec<String>,
    stale_ids: Vec<String>,
    records: Vec<MemoryRecord>,
}

pub fn run_memory_retrieval() -> Result<MemoryRetrievalEvaluationReport> {
    let report = evaluate_memory_retrieval_fixture()?;
    persist_memory_retrieval_report(&report)?;
    Ok(report)
}

fn evaluate_memory_retrieval_fixture() -> Result<MemoryRetrievalEvaluationReport> {
    let fixture: MemoryRetrievalFixture =
        serde_json::from_str(include_str!("../data/evaluation/memory-retrieval-v1.json"))
            .context("invalid embedded memory retrieval fixture")?;
    anyhow::ensure!(fixture.schema_version == 1, "unsupported memory retrieval fixture schema");
    anyhow::ensure!(fixture.top_k > 0, "memory retrieval fixture top_k must be positive");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_secs();
    let mut case_reports = Vec::new();
    for case in &fixture.cases {
        let store = MemoryStore {
            schema_version: MEMORY_SCHEMA_VERSION,
            revision: 0,
            records: case.records.clone(),
        };
        store.validate()?;
        let retrieved = store.search_ranked(
            &case.query,
            &MemorySearchFilters::default(),
            case.now_unix,
            fixture.top_k,
        );
        let retrieved_ids = retrieved.iter().map(|item| item.record_id.clone()).collect::<Vec<_>>();
        let expected = case.expected_ids.iter().cloned().collect::<std::collections::HashSet<_>>();
        let stale = case.stale_ids.iter().cloned().collect::<std::collections::HashSet<_>>();
        let expired = case.records.iter()
            .filter(|record| record.expires_at_unix.is_some_and(|expires| expires <= case.now_unix))
            .map(|record| record.id.clone())
            .collect::<std::collections::HashSet<_>>();
        let hits = retrieved_ids.iter().filter(|id| expected.contains(*id)).count();
        let stale_results_at_k = retrieved_ids.iter().filter(|id| stale.contains(*id)).count();
        let expired_leakage = retrieved_ids.iter().filter(|id| expired.contains(*id)).count();
        let precision_at_k = hits as f64 / fixture.top_k as f64;
        let recall_at_k = hits as f64 / case.expected_ids.len().max(1) as f64;
        case_reports.push(MemoryRetrievalCaseReport {
            case_id: case.id.clone(),
            expected_ids: case.expected_ids.clone(),
            retrieved_ids,
            precision_at_k,
            recall_at_k,
            stale_results_at_k,
            expired_leakage,
        });
    }

    let case_count = case_reports.len().max(1) as f64;
    let precision_at_k = case_reports.iter().map(|item| item.precision_at_k).sum::<f64>() / case_count;
    let recall_at_k = case_reports.iter().map(|item| item.recall_at_k).sum::<f64>() / case_count;
    let stale_results_at_k = case_reports.iter().map(|item| item.stale_results_at_k).sum();
    let expired_leakage = case_reports.iter().map(|item| item.expired_leakage).sum();

    let deletion_leakage = memory_forgetting_leakage_check(now)?;
    let passed = precision_at_k >= 0.66
        && recall_at_k >= 0.95
        && stale_results_at_k == 0
        && expired_leakage == 0
        && deletion_leakage == 0;
    Ok(MemoryRetrievalEvaluationReport {
        schema_version: 1,
        suite_id: fixture.suite_id,
        suite_version: fixture.suite_version,
        evaluated_at_unix: now,
        top_k: fixture.top_k,
        precision_at_k,
        recall_at_k,
        stale_results_at_k,
        expired_leakage,
        deletion_leakage,
        passed,
        cases: case_reports,
    })
}

fn memory_forgetting_leakage_check(now: u64) -> Result<usize> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "rocksoul-memory-retrieval-eval-{now}-{nonce}-{}",
        std::process::id()
    ));
    let id = MemoryStore::transact(&directory, |store| {
        store.add_user_entry(
            MemoryKind::Semantic,
            "synthetic deletion leakage sentinel".to_owned(),
            vec!["retrieval-evaluation".to_owned()],
            None,
            now.max(1),
        )
    })?;
    MemoryStore::transact(&directory, |store| {
        anyhow::ensure!(store.delete(&id), "evaluation sentinel could not be deleted");
        Ok(())
    })?;
    let loaded = MemoryStore::load(&directory)?;
    let leaked = loaded.search_ranked(
        "synthetic deletion leakage sentinel",
        &MemorySearchFilters::default(),
        now.saturating_add(1),
        usize::MAX,
    ).iter().any(|result| result.record_id == id);
    let _ = fs::remove_dir_all(&directory);
    Ok(if leaked { 1 } else { 0 })
}

fn persist_memory_retrieval_report(report: &MemoryRetrievalEvaluationReport) -> Result<()> {
    let path = AppPaths::discover()?
        .data_dir
        .join("evaluations")
        .join("memory-retrieval")
        .join("latest.json");
    let parent = path.parent().context("memory retrieval report path has no parent")?;
    fs::create_dir_all(parent)?;
    let temporary = path.with_extension("json.tmp");
    let backup = path.with_extension("json.bak");
    fs::write(&temporary, serde_json::to_vec_pretty(report)?)?;
    let moved_previous = if path.exists() {
        if backup.exists() {
            fs::remove_file(&backup)?;
        }
        fs::rename(&path, &backup)?;
        true
    } else {
        false
    };
    if let Err(error) = fs::rename(&temporary, &path) {
        if moved_previous {
            let _ = fs::rename(&backup, &path);
        }
        let _ = fs::remove_file(&temporary);
        return Err(error).with_context(|| format!("could not replace {}", path.display()));
    }
    if moved_previous {
        fs::remove_file(&backup)?;
    }
    Ok(())
}

pub fn latest_memory_retrieval() -> Result<Option<MemoryRetrievalEvaluationReport>> {
    let path = AppPaths::discover()?
        .data_dir
        .join("evaluations")
        .join("memory-retrieval")
        .join("latest.json");
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(&path).with_context(|| format!("could not read {}", path.display()))?;
    Ok(Some(serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid memory retrieval report at {}", path.display()))?))
}

pub fn print_memory_retrieval_report(report: &MemoryRetrievalEvaluationReport) {
    println!(
        "Memory retrieval evaluation — {} v{}",
        report.suite_id, report.suite_version
    );
    println!(
        "Result: {} · precision@{} {:.3} · recall@{} {:.3}",
        if report.passed { "PASS" } else { "FAIL" },
        report.top_k,
        report.precision_at_k,
        report.top_k,
        report.recall_at_k
    );
    println!(
        "Stale results: {} · expired leakage: {} · deletion leakage: {}",
        report.stale_results_at_k, report.expired_leakage, report.deletion_leakage
    );
    for case in &report.cases {
        println!(
            "  [{}] precision {:.3} · recall {:.3} · retrieved {:?}",
            case.case_id, case.precision_at_k, case.recall_at_k, case.retrieved_ids
        );
    }
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
    let parent = path
        .parent()
        .context("evaluation report path has no parent")?;
    fs::create_dir_all(parent).with_context(|| format!("could not create {}", parent.display()))?;
    let bytes = serde_json::to_vec_pretty(report)?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, bytes)
        .with_context(|| format!("could not write {}", temporary.display()))?;

    // Windows rename does not replace an existing destination. Preserve the
    // previous report until the new file is in place, then remove the backup.
    let backup = path.with_extension("json.bak");
    let moved_previous = if path.exists() {
        if backup.exists() {
            fs::remove_file(&backup)
                .with_context(|| format!("could not remove {}", backup.display()))?;
        }
        fs::rename(&path, &backup)
            .with_context(|| format!("could not back up {}", path.display()))?;
        true
    } else {
        false
    };

    if let Err(error) = fs::rename(&temporary, &path) {
        if moved_previous {
            let _ = fs::rename(&backup, &path);
        }
        return Err(error).with_context(|| format!("could not replace {}", path.display()));
    }
    if moved_previous {
        fs::remove_file(&backup)
            .with_context(|| format!("could not remove {}", backup.display()))?;
    }
    Ok(())
}

fn latest_report_path() -> Result<PathBuf> {
    Ok(AppPaths::discover()?
        .data_dir
        .join("evaluations")
        .join("core")
        .join("latest.json"))
}

fn check(id: &str, passed: bool, detail: &str) -> EvaluationCheck {
    EvaluationCheck {
        id: id.to_owned(),
        passed,
        detail: detail.to_owned(),
    }
}

pub fn print_report(report: &EvaluationReport) {
    println!(
        "Rocksoul evaluation — {} v{}",
        report.suite_id, report.suite_version
    );
    println!(
        "Runtime: {} / {} · Rocksoul {}",
        report.platform, report.architecture, report.rocksoul_version
    );
    println!(
        "Result: {} ({}/{} passed, {} failed)",
        if report.passed { "PASS" } else { "FAIL" },
        report.passed_count(),
        report.checks.len(),
        report.failed_count()
    );
    for item in &report.checks {
        println!(
            "  [{}] {} — {}",
            if item.passed { "PASS" } else { "FAIL" },
            item.id,
            item.detail
        );
    }
}

#[cfg(test)]
mod tests {
    use super::EvaluationReport;

    #[test]
    fn offline_memory_retrieval_fixture_meets_quality_and_privacy_gates() {
        let report = super::evaluate_memory_retrieval_fixture().unwrap();
        assert!(report.passed, "memory retrieval report failed: {report:?}");
        assert!(report.precision_at_k >= 0.66);
        assert!(report.recall_at_k >= 0.95);
        assert_eq!(report.stale_results_at_k, 0);
        assert_eq!(report.expired_leakage, 0);
        assert_eq!(report.deletion_leakage, 0);
    }

    #[test]
    fn evaluation_report_counts_failed_checks_without_averaging_them_away() {
        let report = EvaluationReport {
            schema_version: 1,
            suite_id: "test".into(),
            suite_version: "1".into(),
            rocksoul_version: "0.1.0".into(),
            platform: "test".into(),
            architecture: "test".into(),
            evaluated_at_unix: 1,
            passed: false,
            checks: vec![
                super::check("safe", true, "ok"),
                super::check("unsafe", false, "veto"),
            ],
        };
        assert_eq!(report.passed_count(), 1);
        assert_eq!(report.failed_count(), 1);
        assert!(!report.passed);
        let encoded = serde_json::to_vec(&report).unwrap();
        let decoded: EvaluationReport = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.failed_count(), 1);
    }
}
