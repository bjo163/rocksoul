//! Narrow, user-approved workspace text-artifact capability.
//!
//! This module intentionally supports only create-only UTF-8 .txt artifacts
//! with a flat filename. It does not expose general filesystem access.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow, bail};

use crate::core::CancellationToken;

pub const MAX_ARTIFACT_BYTES: usize = 16 * 1024;
pub const MAX_FILENAME_BYTES: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceArtifactProposal {
    pub filename: String,
    pub content: String,
}

impl WorkspaceArtifactProposal {
    pub fn validate(&self) -> Result<()> {
        let name = self.filename.as_str();
        if name.is_empty()
            || name.len() > MAX_FILENAME_BYTES
            || !name.ends_with(".txt")
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
            || name == ".txt"
            || name.starts_with('.')
            || name.contains("..")
        {
            bail!("artifact filename must be a simple ASCII .txt basename");
        }
        if self.content.is_empty() || self.content.len() > MAX_ARTIFACT_BYTES {
            bail!("artifact content must be between 1 and {MAX_ARTIFACT_BYTES} bytes");
        }
        Ok(())
    }

    /// Exact approval phrase binds consent to the proposed filename.
    pub fn approval_phrase(&self) -> String {
        format!("WRITE {}", self.filename)
    }

    /// Stable idempotency identity for this exact target/content pair.
    pub fn operation_id(&self) -> String {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in self
            .filename
            .bytes()
            .chain(std::iter::once(0))
            .chain(self.content.bytes())
        {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        format!("workspace:{}:{hash:016x}", self.filename)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserApproval {
    filename: String,
}

impl UserApproval {
    pub fn from_typed_confirmation(
        proposal: &WorkspaceArtifactProposal,
        typed_confirmation: &str,
    ) -> Result<Self> {
        proposal.validate()?;
        if typed_confirmation.trim() != proposal.approval_phrase() {
            bail!("approval phrase did not match the exact proposed artifact");
        }
        Ok(Self {
            filename: proposal.filename.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactOutcome {
    Created {
        path: PathBuf,
        bytes: usize,
        observed_content: String,
    },
    AlreadyPresent {
        path: PathBuf,
        bytes: usize,
        observed_content: String,
    },
}

#[derive(Clone, Debug)]
pub struct WorkspaceArtifactService {
    workspace_root: PathBuf,
}

impl WorkspaceArtifactService {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }

    /// Retry only transient filesystem errors. The operation is create-only,
    /// so replaying after an ambiguous publish is safe and resolves as AlreadyPresent.
    pub fn execute_approved_with_retry(
        &self,
        proposal: &WorkspaceArtifactProposal,
        approval: &UserApproval,
        cancellation: &CancellationToken,
    ) -> Result<ArtifactOutcome> {
        const MAX_ATTEMPTS: u8 = 3;
        const BASE_BACKOFF_MS: u64 = 20;

        for attempt in 1..=MAX_ATTEMPTS {
            cancellation.check()?;
            match self.execute_approved(proposal, approval, cancellation) {
                Ok(outcome) => return Ok(outcome),
                Err(error) if is_retryable_io_error(&error) && attempt < MAX_ATTEMPTS => {
                    std::thread::sleep(Duration::from_millis(BASE_BACKOFF_MS * u64::from(attempt)));
                }
                Err(error) => return Err(error),
            }
        }
        unreachable!("the bounded retry loop always returns or exits with an error")
    }

    /// Publish a create-only artifact after exact user approval, then reopen
    /// the final path and compare the observed bytes with the proposal.
    ///
    /// Existing identical content is treated as an idempotent replay. A
    /// conflicting existing file is never overwritten.
    pub fn execute_approved(
        &self,
        proposal: &WorkspaceArtifactProposal,
        approval: &UserApproval,
        cancellation: &CancellationToken,
    ) -> Result<ArtifactOutcome> {
        cancellation.check()?;
        proposal.validate()?;
        if approval.filename != proposal.filename {
            bail!("approval does not authorize this artifact");
        }

        self.prepare_workspace()?;
        let root = fs::canonicalize(&self.workspace_root)
            .context("could not canonicalize workspace root")?;
        let target = root.join(&proposal.filename);

        match fs::symlink_metadata(&target) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    bail!("existing artifact target is not a regular file");
                }
                let observed = fs::read(&target).context("could not read existing artifact")?;
                if observed != proposal.content.as_bytes() {
                    bail!("artifact already exists with different content; refusing overwrite");
                }
                Ok(ArtifactOutcome::AlreadyPresent {
                    path: target,
                    bytes: observed.len(),
                    observed_content: String::from_utf8(observed)
                        .context("existing artifact is not valid UTF-8")?,
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.create_and_verify(&root, &target, proposal, cancellation)
            }
            Err(error) => Err(error).context("could not inspect artifact target"),
        }
    }

    fn prepare_workspace(&self) -> Result<()> {
        match fs::symlink_metadata(&self.workspace_root) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    bail!("workspace root must be a real directory, not a symlink");
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&self.workspace_root)
                    .context("could not create workspace directory")?;
            }
            Err(error) => return Err(error).context("could not inspect workspace directory"),
        }
        Ok(())
    }

    fn create_and_verify(
        &self,
        root: &Path,
        target: &Path,
        proposal: &WorkspaceArtifactProposal,
        cancellation: &CancellationToken,
    ) -> Result<ArtifactOutcome> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock is before the Unix epoch")?
            .as_nanos();
        let temporary = root.join(format!(".rocksoul-{}-{nonce}.tmp", std::process::id()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .context("could not create temporary artifact")?;
        file.write_all(proposal.content.as_bytes())
            .context("could not write temporary artifact")?;
        file.sync_all()
            .context("could not sync temporary artifact")?;
        drop(file);

        // Cancellation is checked before the irreversible publication step.
        if let Err(error) = cancellation.check() {
            let _ = fs::remove_file(&temporary);
            return Err(error.into());
        }

        // Hard-link publication is create-only: it never replaces a target.
        let published = match fs::hard_link(&temporary, target) {
            Ok(()) => true,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => false,
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                return Err(error).context("could not publish artifact without overwrite");
            }
        };
        let _ = fs::remove_file(&temporary);

        let metadata =
            fs::symlink_metadata(target).context("could not inspect published artifact")?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!("published artifact is not a regular file");
        }
        let observed = fs::read(target).context("could not reopen published artifact")?;
        if observed != proposal.content.as_bytes() {
            bail!("independent artifact read-back did not match proposed bytes");
        }

        if published {
            Ok(ArtifactOutcome::Created {
                path: target.to_path_buf(),
                bytes: observed.len(),
                observed_content: String::from_utf8(observed.clone())
                    .context("published artifact is not valid UTF-8")?,
            })
        } else {
            // A concurrent identical create is a safe replay; conflicting
            // content remains a hard failure.
            if observed == proposal.content.as_bytes() {
                Ok(ArtifactOutcome::AlreadyPresent {
                    path: target.to_path_buf(),
                    bytes: observed.len(),
                    observed_content: String::from_utf8(observed.clone())
                        .context("concurrent artifact is not valid UTF-8")?,
                })
            } else {
                Err(anyhow!(
                    "artifact target was concurrently created with different content"
                ))
            }
        }
    }
}

fn is_retryable_io_error(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io_error| {
                matches!(
                    io_error.kind(),
                    std::io::ErrorKind::Interrupted
                        | std::io::ErrorKind::WouldBlock
                        | std::io::ErrorKind::TimedOut
                )
            })
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        core::{
            CancellationToken, TurnEvent, TurnState, VerificationEvidence, VerificationOutcome,
            VerificationRequest, verify_turn_and_award_xp,
        },
        rid::{RuhIdentity, RuhProfile, XpRecordOutcome},
    };

    use super::{
        ArtifactOutcome, UserApproval, WorkspaceArtifactProposal, WorkspaceArtifactService,
        is_retryable_io_error,
    };

    fn workspace() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("rocksoul-workspace-test-{nonce}"))
    }

    fn proposal(filename: &str, content: &str) -> WorkspaceArtifactProposal {
        WorkspaceArtifactProposal {
            filename: filename.into(),
            content: content.into(),
        }
    }

    fn approve(proposal: &WorkspaceArtifactProposal) -> UserApproval {
        UserApproval::from_typed_confirmation(proposal, &proposal.approval_phrase()).unwrap()
    }

    #[test]
    fn classifies_only_transient_filesystem_errors_for_retry() {
        let transient = anyhow::Error::new(std::io::Error::from(std::io::ErrorKind::WouldBlock));
        let timeout = anyhow::Error::new(std::io::Error::from(std::io::ErrorKind::TimedOut));
        let permanent =
            anyhow::Error::new(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
        assert!(is_retryable_io_error(&transient));
        assert!(is_retryable_io_error(&timeout));
        assert!(!is_retryable_io_error(&permanent));
    }

    #[test]
    fn creates_artifact_and_independently_reads_back_bytes() {
        let root = workspace();
        let item = proposal("note.txt", "verified output");
        let outcome = WorkspaceArtifactService::new(&root)
            .execute_approved(&item, &approve(&item), &CancellationToken::default())
            .unwrap();
        assert!(matches!(
            outcome,
            ArtifactOutcome::Created { bytes: 15, .. }
        ));
        assert_eq!(fs::read(root.join("note.txt")).unwrap(), b"verified output");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn identical_replay_is_idempotent_and_conflict_never_overwrites() {
        let root = workspace();
        let service = WorkspaceArtifactService::new(&root);
        let item = proposal("note.txt", "first");
        assert!(matches!(
            service
                .execute_approved(&item, &approve(&item), &CancellationToken::default())
                .unwrap(),
            ArtifactOutcome::Created { .. }
        ));
        assert!(matches!(
            service
                .execute_approved(&item, &approve(&item), &CancellationToken::default())
                .unwrap(),
            ArtifactOutcome::AlreadyPresent { .. }
        ));
        let conflict = proposal("note.txt", "second");
        assert!(
            service
                .execute_approved(
                    &conflict,
                    &approve(&conflict),
                    &CancellationToken::default()
                )
                .is_err()
        );
        assert_eq!(fs::read(root.join("note.txt")).unwrap(), b"first");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn operation_identity_is_stable_for_exact_target_and_content() {
        let first = proposal("note.txt", "same content");
        let replay = proposal("note.txt", "same content");
        let changed = proposal("note.txt", "different content");
        let other_target = proposal("other.txt", "same content");
        assert_eq!(first.operation_id(), replay.operation_id());
        assert_ne!(first.operation_id(), changed.operation_id());
        assert_ne!(first.operation_id(), other_target.operation_id());
    }

    #[test]
    fn rejects_unapproved_or_unsafe_proposals() {
        let root = workspace();
        let item = proposal("../escape.txt", "no");
        assert!(item.validate().is_err());
        let item = proposal("safe.txt", "content");
        let wrong = UserApproval::from_typed_confirmation(&item, "WRITE other.txt");
        assert!(wrong.is_err());
        assert!(!root.exists());
    }

    #[test]
    fn cancelled_operation_does_not_publish_artifact() {
        let root = workspace();
        let item = proposal("cancelled.txt", "content");
        let cancellation = CancellationToken::default();
        cancellation.cancel();
        assert!(
            WorkspaceArtifactService::new(&root)
                .execute_approved(&item, &approve(&item), &cancellation)
                .is_err()
        );
        assert!(!root.exists());
    }

    #[test]
    fn approved_artifact_readback_gates_xp_end_to_end() {
        let root = workspace();
        let item = proposal("verified.txt", "observed artifact content");
        let outcome = WorkspaceArtifactService::new(&root)
            .execute_approved(&item, &approve(&item), &CancellationToken::default())
            .unwrap();
        let (path, observed_content) = match outcome {
            ArtifactOutcome::Created {
                path,
                observed_content,
                ..
            }
            | ArtifactOutcome::AlreadyPresent {
                path,
                observed_content,
                ..
            } => (path, observed_content),
        };
        let is_regular_file = path.is_file();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .max(1);
        let mut profile = RuhProfile::new(RuhIdentity {
            instance_id: format!("rocksoul-workspace-e2e-{now}-{}", std::process::id()),
            created_at_unix: now,
        })
        .unwrap();
        let mut turn = TurnState::new("workspace-e2e", 0);
        turn.apply(TurnEvent::Start).unwrap();
        turn.apply(TurnEvent::PlanAccepted).unwrap();
        turn.apply(TurnEvent::ActionCompleted).unwrap();
        let request = VerificationRequest::new(
            "workspace-e2e",
            "workspace-e2e-verification",
            now,
            1,
            1,
            vec![
                VerificationEvidence::new(
                    "artifact-is-regular-file",
                    "true",
                    if is_regular_file { "true" } else { "false" },
                    path.display().to_string(),
                    true,
                ),
                VerificationEvidence::new(
                    "artifact-readback-matches",
                    item.content.as_str(),
                    observed_content.as_str(),
                    format!("workspace-readback:{}", item.filename),
                    true,
                ),
            ],
        );
        let verification = verify_turn_and_award_xp(&mut turn, &mut profile, request).unwrap();
        assert!(matches!(
            verification,
            VerificationOutcome::Verified {
                xp_record: XpRecordOutcome::Recorded,
                ..
            }
        ));
        assert!(profile.total_xp() > 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_unsupported_extensions_and_oversized_content() {
        assert!(proposal("note.md", "content").validate().is_err());
        assert!(proposal("note.txt", &"x".repeat(16 * 1024 + 1)).validate().is_err());
        assert!(proposal("note.txt", "").validate().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_target_symlink_without_writing_through_it() {
        use std::os::unix::fs::symlink;
        let parent = workspace();
        fs::create_dir_all(&parent).unwrap();
        let outside = parent.join("outside.txt");
        fs::write(&outside, "untouched").unwrap();
        let root = parent.join("workspace");
        fs::create_dir(&root).unwrap();
        symlink(&outside, root.join("note.txt")).unwrap();
        let item = proposal("note.txt", "attacker-controlled target");
        assert!(WorkspaceArtifactService::new(&root)
            .execute_approved(&item, &approve(&item), &CancellationToken::default())
            .is_err());
        assert_eq!(fs::read_to_string(outside).unwrap(), "untouched");
        fs::remove_dir_all(parent).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn rejects_workspace_symlink() {
        use std::os::unix::fs::symlink;
        let parent = workspace();
        fs::create_dir_all(&parent).unwrap();
        let outside = parent.join("outside");
        fs::create_dir(&outside).unwrap();
        let link = parent.join("workspace-link");
        symlink(&outside, &link).unwrap();
        let item = proposal("note.txt", "content");
        assert!(
            WorkspaceArtifactService::new(&link)
                .execute_approved(&item, &approve(&item), &CancellationToken::default())
                .is_err()
        );
        fs::remove_dir_all(parent).unwrap();
    }
}
