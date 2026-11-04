//! Narrow, user-approved workspace text-artifact capability.
//!
//! This module intentionally supports only create-only UTF-8 .txt artifacts
//! with a flat filename. It does not expose general filesystem access.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow, bail};

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
            || !name.bytes().all(|byte| byte.is_ascii_alphanumeric()
                || matches!(byte, b'-' | b'_' | b'.'))
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
    Created { path: PathBuf, bytes: usize },
    AlreadyPresent { path: PathBuf, bytes: usize },
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

    /// Publish a create-only artifact after exact user approval, then reopen
    /// the final path and compare the observed bytes with the proposal.
    ///
    /// Existing identical content is treated as an idempotent replay. A
    /// conflicting existing file is never overwritten.
    pub fn execute_approved(
        &self,
        proposal: &WorkspaceArtifactProposal,
        approval: &UserApproval,
    ) -> Result<ArtifactOutcome> {
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
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.create_and_verify(&root, &target, proposal)
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
        file.sync_all().context("could not sync temporary artifact")?;
        drop(file);

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

        let metadata = fs::symlink_metadata(target).context("could not inspect published artifact")?;
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
            })
        } else {
            // A concurrent identical create is a safe replay; conflicting
            // content remains a hard failure.
            if observed == proposal.content.as_bytes() {
                Ok(ArtifactOutcome::AlreadyPresent {
                    path: target.to_path_buf(),
                    bytes: observed.len(),
                })
            } else {
                Err(anyhow!("artifact target was concurrently created with different content"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        ArtifactOutcome, UserApproval, WorkspaceArtifactProposal, WorkspaceArtifactService,
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
    fn creates_artifact_and_independently_reads_back_bytes() {
        let root = workspace();
        let item = proposal("note.txt", "verified output");
        let outcome = WorkspaceArtifactService::new(&root)
            .execute_approved(&item, &approve(&item))
            .unwrap();
        assert!(matches!(outcome, ArtifactOutcome::Created { bytes: 15, .. }));
        assert_eq!(fs::read(root.join("note.txt")).unwrap(), b"verified output");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn identical_replay_is_idempotent_and_conflict_never_overwrites() {
        let root = workspace();
        let service = WorkspaceArtifactService::new(&root);
        let item = proposal("note.txt", "first");
        assert!(matches!(
            service.execute_approved(&item, &approve(&item)).unwrap(),
            ArtifactOutcome::Created { .. }
        ));
        assert!(matches!(
            service.execute_approved(&item, &approve(&item)).unwrap(),
            ArtifactOutcome::AlreadyPresent { .. }
        ));
        let conflict = proposal("note.txt", "second");
        assert!(service.execute_approved(&conflict, &approve(&conflict)).is_err());
        assert_eq!(fs::read(root.join("note.txt")).unwrap(), b"first");
        fs::remove_dir_all(root).unwrap();
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
        assert!(WorkspaceArtifactService::new(&link)
            .execute_approved(&item, &approve(&item))
            .is_err());
        fs::remove_dir_all(parent).unwrap();
    }
}
