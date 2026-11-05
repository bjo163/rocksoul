use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use anyhow::{Context, Result, bail};

use crate::memory::{MemoryKind, MemoryStore};

const PROPOSAL_SCHEMA_VERSION: u16 = 1;
const PROPOSAL_FILE: &str = "proposals.v1.json";
const PROPOSAL_LOCK_FILE: &str = ".proposals.lock";
const LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const LOCK_RETRY: Duration = Duration::from_millis(25);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Pending,
    Approved,
    Denied,
    Cancelled,
}

impl std::fmt::Display for ProposalStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Denied => "denied",
            Self::Cancelled => "cancelled",
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MemoryProposal {
    pub id: String,
    pub kind: MemoryKind,
    pub content: String,
    pub tags: Vec<String>,
    pub source_turn_reference: String,
    pub provenance: String,
    pub created_at_unix: u64,
    pub expires_at_unix: Option<u64>,
    pub fingerprint: String,
    pub status: ProposalStatus,
    pub decided_at_unix: Option<u64>,
    pub memory_record_id: Option<String>,
}

impl MemoryProposal {
    pub fn approval_phrase(&self) -> String {
        format!("APPROVE {} {}", self.id, self.fingerprint)
    }
}

#[derive(Clone, Debug)]
pub struct MemoryProposalCandidate {
    pub kind: MemoryKind,
    pub content: String,
    pub tags: Vec<String>,
    pub source_turn_reference: String,
    pub provenance: String,
    pub expires_at_unix: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ProposalStore {
    schema_version: u16,
    proposals: Vec<MemoryProposal>,
}

impl Default for ProposalStore {
    fn default() -> Self {
        Self {
            schema_version: PROPOSAL_SCHEMA_VERSION,
            proposals: Vec::new(),
        }
    }
}

impl ProposalStore {
    fn validate(&self) -> Result<()> {
        if self.schema_version != PROPOSAL_SCHEMA_VERSION {
            bail!(
                "unsupported memory proposal schema version {}",
                self.schema_version
            );
        }
        let mut ids = HashSet::new();
        for proposal in &self.proposals {
            if proposal.id.trim().is_empty()
                || proposal.content.trim().is_empty()
                || proposal.source_turn_reference.trim().is_empty()
                || proposal.provenance.trim().is_empty()
                || !ids.insert(proposal.id.as_str())
            {
                bail!("memory proposal store contains invalid or duplicate proposals");
            }
            validate_candidate(
                proposal.kind,
                &proposal.content,
                &proposal.tags,
                &proposal.source_turn_reference,
                &proposal.provenance,
                proposal.expires_at_unix,
                proposal.created_at_unix,
            )?;
            if fingerprint(proposal) != proposal.fingerprint {
                bail!(
                    "memory proposal {} payload fingerprint mismatch; refusing edited proposal",
                    proposal.id
                );
            }
            match proposal.status {
                ProposalStatus::Pending
                    if proposal.decided_at_unix.is_some()
                        || proposal.memory_record_id.is_some() =>
                {
                    bail!("pending memory proposal has decision metadata");
                }
                ProposalStatus::Approved
                    if proposal.decided_at_unix.is_none()
                        || proposal.memory_record_id.is_none() =>
                {
                    bail!("approved memory proposal is missing its persisted decision");
                }
                ProposalStatus::Denied | ProposalStatus::Cancelled
                    if proposal.decided_at_unix.is_none()
                        || proposal.memory_record_id.is_some() =>
                {
                    bail!("denied/cancelled memory proposal has invalid decision metadata");
                }
                _ => {}
            }
        }
        Ok(())
    }
}

pub fn create(
    directory: &Path,
    candidate: MemoryProposalCandidate,
    now: u64,
) -> Result<MemoryProposal> {
    let MemoryProposalCandidate {
        kind,
        content,
        tags,
        source_turn_reference,
        provenance,
        expires_at_unix,
    } = candidate;
    validate_candidate(
        kind,
        &content,
        &tags,
        &source_turn_reference,
        &provenance,
        expires_at_unix,
        now,
    )?;
    let _lock = acquire_lock(directory)?;
    let mut store = load_unlocked(directory)?;
    let id = format!(
        "proposal-{now}-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    );
    let mut proposal = MemoryProposal {
        id,
        kind,
        content: content.trim().to_owned(),
        tags: normalize_tags(tags),
        source_turn_reference,
        provenance,
        created_at_unix: now,
        expires_at_unix,
        fingerprint: String::new(),
        status: ProposalStatus::Pending,
        decided_at_unix: None,
        memory_record_id: None,
    };
    proposal.fingerprint = fingerprint(&proposal);
    if store
        .proposals
        .iter()
        .any(|existing| existing.fingerprint == proposal.fingerprint)
    {
        bail!("duplicate memory proposal for this exact candidate and source turn");
    }
    store.proposals.push(proposal.clone());
    save_unlocked(directory, &store)?;
    Ok(proposal)
}

pub fn list(directory: &Path) -> Result<Vec<MemoryProposal>> {
    let _lock = acquire_lock(directory)?;
    let mut proposals = load_unlocked(directory)?.proposals;
    proposals.sort_by(|left, right| {
        right
            .created_at_unix
            .cmp(&left.created_at_unix)
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(proposals)
}

pub fn get_pending(directory: &Path, id: &str) -> Result<MemoryProposal> {
    let _lock = acquire_lock(directory)?;
    let store = load_unlocked(directory)?;
    let proposal = store
        .proposals
        .iter()
        .find(|proposal| proposal.id == id)
        .with_context(|| format!("memory proposal not found: {id}"))?;
    if proposal.status != ProposalStatus::Pending {
        bail!("memory proposal {id} is already {}", proposal.status);
    }
    Ok(proposal.clone())
}

pub fn approve(directory: &Path, id: &str, confirmation: &str, now: u64) -> Result<String> {
    let _lock = acquire_lock(directory)?;
    let mut proposals = load_unlocked(directory)?;
    let proposal = proposals
        .proposals
        .iter_mut()
        .find(|proposal| proposal.id == id)
        .with_context(|| format!("memory proposal not found: {id}"))?;
    if proposal.status != ProposalStatus::Pending {
        bail!("memory proposal {id} is already {}", proposal.status);
    }
    if fingerprint(proposal) != proposal.fingerprint {
        bail!("memory proposal payload changed; refusing approval");
    }
    if confirmation.trim() != proposal.approval_phrase() {
        bail!("approval phrase does not match this exact proposal");
    }

    let proposal_snapshot = proposal.clone();
    let source = format!("model_proposal:{}", proposal_snapshot.id);
    let memory_record_id = MemoryStore::transact(directory, |memory| {
        if let Some(existing) = memory.records.iter().find(|record| record.source == source) {
            return Ok(existing.id.clone());
        }
        let record_id = memory.add_user_entry(
            proposal_snapshot.kind,
            proposal_snapshot.content.clone(),
            proposal_snapshot.tags.clone(),
            proposal_snapshot.expires_at_unix,
            now,
        )?;
        let record = memory
            .records
            .iter_mut()
            .find(|record| record.id == record_id)
            .context("newly added memory record disappeared before commit")?;
        record.source = source.clone();
        Ok(record_id)
    })?;

    proposal.status = ProposalStatus::Approved;
    proposal.decided_at_unix = Some(now);
    proposal.memory_record_id = Some(memory_record_id.clone());
    save_unlocked(directory, &proposals)?;
    Ok(memory_record_id)
}

pub fn deny(directory: &Path, id: &str, now: u64) -> Result<()> {
    decide_without_memory(directory, id, now, ProposalStatus::Denied)
}

pub fn cancel(directory: &Path, id: &str, now: u64) -> Result<()> {
    decide_without_memory(directory, id, now, ProposalStatus::Cancelled)
}

fn decide_without_memory(
    directory: &Path,
    id: &str,
    now: u64,
    status: ProposalStatus,
) -> Result<()> {
    let _lock = acquire_lock(directory)?;
    let mut store = load_unlocked(directory)?;
    let proposal = store
        .proposals
        .iter_mut()
        .find(|proposal| proposal.id == id)
        .with_context(|| format!("memory proposal not found: {id}"))?;
    if proposal.status != ProposalStatus::Pending {
        bail!("memory proposal {id} is already {}", proposal.status);
    }
    proposal.status = status;
    proposal.decided_at_unix = Some(now);
    save_unlocked(directory, &store)
}

fn validate_candidate(
    kind: MemoryKind,
    content: &str,
    tags: &[String],
    source_turn_reference: &str,
    provenance: &str,
    expires_at_unix: Option<u64>,
    now: u64,
) -> Result<()> {
    if kind == MemoryKind::Derived {
        bail!(
            "model-proposed derived memory is unsupported; derived records must be computed by trusted code"
        );
    }
    if content.trim().is_empty() || content.len() > 16_384 || now == 0 {
        bail!("proposal content must contain 1–16384 UTF-8 bytes and a valid timestamp");
    }
    if source_turn_reference.trim().is_empty()
        || source_turn_reference.len() > 256
        || source_turn_reference.chars().any(char::is_control)
    {
        bail!("a valid source-turn reference (maximum 256 characters) is required");
    }
    if provenance.trim().is_empty()
        || provenance.len() > 128
        || provenance.chars().any(char::is_control)
    {
        bail!("proposal provenance must contain 1–128 printable characters");
    }
    if tags
        .iter()
        .any(|tag| tag.trim().is_empty() || tag.len() > 64)
    {
        bail!("proposal tags must contain 1–64 characters each");
    }
    if expires_at_unix.is_some() && kind != MemoryKind::Working {
        bail!("expiry suggestions are currently supported only for working memory");
    }
    if expires_at_unix.is_some_and(|expiry| expiry <= now) {
        bail!("proposal expiry must be in the future");
    }
    if let Some(reason) = sensitive_content_reason(content) {
        bail!(
            "proposal rejected by privacy guard ({reason}); remove sensitive data before proposing memory"
        );
    }
    Ok(())
}

fn sensitive_content_reason(content: &str) -> Option<&'static str> {
    let normalized = content.to_ascii_lowercase();
    let secret_markers = [
        "-----begin private key-----",
        "-----begin rsa private key-----",
        "-----begin openssh private key-----",
        "github_pat_",
        "ghp_",
        "gho_",
        "ghs_",
        "sk-",
        "api_key=",
        "api-key=",
        "password=",
        "password:",
        "password is ",
        "api key is ",
        "access token",
        "secret key",
        "secret=",
        "authorization: bearer",
        "bearer ",
    ];
    if secret_markers
        .iter()
        .any(|marker| normalized.contains(marker))
    {
        return Some("credential_or_secret");
    }
    if content.contains("AKIA")
        && content.as_bytes().windows(20).any(|window| {
            window.starts_with(b"AKIA")
                && window[4..].iter().all(|byte| byte.is_ascii_alphanumeric())
        })
    {
        return Some("cloud_access_key");
    }
    let sensitive_categories = [
        "medical diagnosis",
        "medical record",
        "health condition",
        "my medication",
        "religious affiliation",
        "political affiliation",
        "sexual orientation",
        "bank account number",
        "credit card number",
        "passport number",
        "national id number",
        "home address",
        "precise location",
        "criminal record",
        "ethnicity:",
        "race:",
    ];
    if sensitive_categories
        .iter()
        .any(|marker| normalized.contains(marker))
    {
        return Some("sensitive_personal_category");
    }
    None
}

fn fingerprint(proposal: &MemoryProposal) -> String {
    let payload = serde_json::to_vec(&(
        proposal.kind,
        &proposal.content,
        &proposal.tags,
        &proposal.source_turn_reference,
        &proposal.provenance,
        proposal.expires_at_unix,
    ))
    .expect("serializing a memory proposal payload cannot fail");
    let digest = Sha256::digest(payload);
    format!("{digest:x}")
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut normalized = tags
        .into_iter()
        .map(|tag| tag.trim().to_owned())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn proposal_path(directory: &Path) -> PathBuf {
    directory.join(PROPOSAL_FILE)
}

fn load_unlocked(directory: &Path) -> Result<ProposalStore> {
    let path = proposal_path(directory);
    if !path.is_file() {
        let backup = directory.join(format!("{PROPOSAL_FILE}.bak"));
        if backup.is_file() {
            fs::rename(&backup, &path).with_context(|| {
                format!("could not recover proposal store from {}", backup.display())
            })?;
        } else {
            return Ok(ProposalStore::default());
        }
    }
    let bytes = fs::read(&path).with_context(|| format!("could not read {}", path.display()))?;
    let store: ProposalStore = serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid memory proposal store {}", path.display()))?;
    store.validate()?;
    Ok(store)
}

fn save_unlocked(directory: &Path, store: &ProposalStore) -> Result<()> {
    store.validate()?;
    fs::create_dir_all(directory)?;
    let path = proposal_path(directory);
    let temporary = directory.join(format!(".{PROPOSAL_FILE}.{}.tmp", std::process::id()));
    let backup = directory.join(format!("{PROPOSAL_FILE}.bak"));
    let bytes = serde_json::to_vec_pretty(store)?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .with_context(|| {
            format!(
                "could not create proposal temp file {}",
                temporary.display()
            )
        })?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);

    let had_previous = path.exists();
    if had_previous {
        if backup.exists() {
            fs::remove_file(&backup)?;
        }
        fs::rename(&path, &backup).with_context(|| {
            format!(
                "could not preserve previous proposal store {}",
                path.display()
            )
        })?;
    }
    if let Err(error) = fs::rename(&temporary, &path) {
        if had_previous {
            let _ = fs::rename(&backup, &path);
        }
        let _ = fs::remove_file(&temporary);
        return Err(error)
            .with_context(|| format!("could not publish proposal store {}", path.display()));
    }
    Ok(())
}

fn acquire_lock(directory: &Path) -> Result<File> {
    fs::create_dir_all(directory)?;
    let path = directory.join(PROPOSAL_LOCK_FILE);
    let started = Instant::now();
    let file = loop {
        match OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
        {
            Ok(file) => break file,
            Err(error) if lock_contention(&error) => {
                if started.elapsed() >= LOCK_TIMEOUT {
                    bail!(
                        "memory proposal store is busy; timed out opening {}",
                        path.display()
                    );
                }
                thread::sleep(LOCK_RETRY);
            }
            Err(error) => {
                return Err(error).with_context(|| format!("could not open {}", path.display()));
            }
        }
    };
    loop {
        match file.try_lock_exclusive() {
            Ok(()) => return Ok(file),
            Err(error) if lock_contention(&error) => {
                if started.elapsed() >= LOCK_TIMEOUT {
                    bail!(
                        "memory proposal store is busy; timed out locking {}",
                        path.display()
                    );
                }
                thread::sleep(LOCK_RETRY);
            }
            Err(error) => {
                return Err(error).with_context(|| format!("could not lock {}", path.display()));
            }
        }
    }
}

fn lock_contention(error: &std::io::Error) -> bool {
    if error.kind() == ErrorKind::WouldBlock {
        return true;
    }
    #[cfg(windows)]
    {
        matches!(error.raw_os_error(), Some(32 | 33))
    }
    #[cfg(not(windows))]
    {
        false
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
        MemoryProposalCandidate, ProposalStatus, approve, cancel, create, deny, get_pending, list,
    };

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rocksoul-proposal-test-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn candidate(
        kind: crate::memory::MemoryKind,
        content: &str,
        source_turn_reference: &str,
    ) -> MemoryProposalCandidate {
        MemoryProposalCandidate {
            kind,
            content: content.into(),
            tags: vec!["project".into()],
            source_turn_reference: source_turn_reference.into(),
            provenance: "test-model".into(),
            expires_at_unix: None,
        }
    }

    fn create_one(directory: &std::path::Path) -> super::MemoryProposal {
        create(
            directory,
            candidate(
                crate::memory::MemoryKind::Semantic,
                "The project uses a single main branch",
                "turn-42",
            ),
            100,
        )
        .unwrap()
    }

    #[test]
    fn approval_persists_exact_candidate_with_immutable_provenance_and_rejects_replay() {
        let directory = temp_dir();
        let proposal = create_one(&directory);
        let memory_id =
            approve(&directory, &proposal.id, &proposal.approval_phrase(), 101).unwrap();
        let memory = crate::memory::MemoryStore::load(&directory).unwrap();
        assert_eq!(memory.records.len(), 1);
        assert_eq!(memory.records[0].id, memory_id);
        assert_eq!(
            memory.records[0].source,
            format!("model_proposal:{}", proposal.id)
        );
        assert!(memory.records[0].user_approved);
        assert!(approve(&directory, &proposal.id, &proposal.approval_phrase(), 102).is_err());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn denial_and_cancellation_never_create_memory_records() {
        let directory = temp_dir();
        let denied = create_one(&directory);
        deny(&directory, &denied.id, 101).unwrap();
        let cancelled = create(
            &directory,
            candidate(
                crate::memory::MemoryKind::Episodic,
                "A cancelled candidate",
                "turn-43",
            ),
            100,
        )
        .unwrap();
        cancel(&directory, &cancelled.id, 101).unwrap();
        assert_eq!(
            crate::memory::MemoryStore::load(&directory)
                .unwrap()
                .records
                .len(),
            0
        );
        assert_eq!(
            list(&directory)
                .unwrap()
                .iter()
                .filter(|item| item.status == ProposalStatus::Denied)
                .count(),
            1
        );
        assert!(get_pending(&directory, &cancelled.id).is_err());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn duplicate_candidates_are_rejected() {
        let directory = temp_dir();
        create_one(&directory);
        assert!(
            create(
                &directory,
                candidate(
                    crate::memory::MemoryKind::Semantic,
                    "The project uses a single main branch",
                    "turn-42",
                ),
                100,
            )
            .is_err()
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn modified_payload_is_rejected_before_approval() {
        let directory = temp_dir();
        let proposal = create_one(&directory);
        let path = directory.join("proposals.v1.json");
        let mut store: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        store["proposals"][0]["content"] =
            serde_json::Value::String("edited after proposal".into());
        fs::write(&path, serde_json::to_vec(&store).unwrap()).unwrap();
        assert!(approve(&directory, &proposal.id, &proposal.approval_phrase(), 101).is_err());
        assert_eq!(
            crate::memory::MemoryStore::load(&directory)
                .unwrap()
                .records
                .len(),
            0
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn credentials_and_sensitive_categories_are_rejected() {
        let directory = temp_dir();
        for content in [
            "api_key=sk-live-example",
            "My medical diagnosis is confidential",
        ] {
            assert!(
                create(
                    &directory,
                    candidate(crate::memory::MemoryKind::Semantic, content, "turn-secret",)
                    100,
                )
                .is_err()
            );
        }
        assert!(list(&directory).unwrap().is_empty());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn failed_memory_persistence_keeps_proposal_pending() {
        let directory = temp_dir();
        let proposal = create_one(&directory);
        fs::write(
            directory.join("store.v3.json"),
            br#"{"schema_version":99,"revision":0,"records":[]}"#,
        )
        .unwrap();
        assert!(approve(&directory, &proposal.id, &proposal.approval_phrase(), 101).is_err());
        assert_eq!(
            get_pending(&directory, &proposal.id).unwrap().status,
            ProposalStatus::Pending
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
