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

use crate::{
    core::{VerificationReceipt, normalize_task_claim},
    memory::{MemoryClaimStatus, MemoryKind, MemoryStore},
};

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
    pub claim_status: MemoryClaimStatus,
    pub verification_receipt_id: Option<String>,
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
    pub claim_status: MemoryClaimStatus,
    pub verification_receipt_id: Option<String>,
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
            if proposal.id.trim().is_empty() || !ids.insert(proposal.id.as_str()) {
                bail!("memory proposal store contains invalid or duplicate proposals");
            }
            if proposal.source_turn_reference.trim().is_empty()
                || proposal.provenance.trim().is_empty()
                || proposal.fingerprint.len() != 64
                || !proposal
                    .fingerprint
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit())
            {
                bail!("memory proposal provenance or SHA-256 fingerprint is invalid");
            }
            match proposal.status {
                ProposalStatus::Pending => {
                    if proposal.decided_at_unix.is_some() || proposal.memory_record_id.is_some() {
                        bail!("pending memory proposal has decision metadata");
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
                }
                ProposalStatus::Approved => {
                    if proposal.decided_at_unix.is_none()
                        || proposal.memory_record_id.is_none()
                        || !proposal.content.is_empty()
                        || !proposal.tags.is_empty()
                    {
                        bail!(
                            "approved proposal must be content-scrubbed and retain decision metadata"
                        );
                    }
                }
                ProposalStatus::Denied | ProposalStatus::Cancelled => {
                    if proposal.decided_at_unix.is_none()
                        || proposal.memory_record_id.is_some()
                        || !proposal.content.is_empty()
                        || !proposal.tags.is_empty()
                    {
                        bail!(
                            "denied/cancelled proposal must be content-scrubbed and have valid decision metadata"
                        );
                    }
                }
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
        claim_status,
        verification_receipt_id,
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
    if !matches!(claim_status, MemoryClaimStatus::Inference | MemoryClaimStatus::UnverifiedTaskOutcome) {
        bail!("model proposals may only be classified as inference or unverified task outcome");
    }
    if verification_receipt_id.as_ref().is_some_and(|id| {
        id.trim().is_empty() || id.len() > 256 || id.chars().any(char::is_control)
    }) {
        bail!("verification receipt ID is invalid");
    }
    if verification_receipt_id.is_some() && claim_status != MemoryClaimStatus::UnverifiedTaskOutcome {
        bail!("verification receipts may only be attached to task-outcome claims");
    }
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
        claim_status,
        verification_receipt_id,
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
    approve_with_receipts(directory, None, id, confirmation, now)
}

pub fn approve_with_receipts(
    directory: &Path,
    receipts_directory: Option<&Path>,
    id: &str,
    confirmation: &str,
    now: u64,
) -> Result<String> {
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
    let receipt_candidate = proposal_snapshot
        .verification_receipt_id
        .as_deref()
        .and_then(|receipt_id| receipts_directory.and_then(|directory| {
            load_verified_receipt(directory, receipt_id, &proposal_snapshot.content, now)
                .ok()
                .flatten()
        }));
    let memory_record_id = MemoryStore::transact(directory, |memory| {
        if let Some(existing) = memory.records.iter().find(|record| record.source == source) {
            return Ok(existing.id.clone());
        }
        let receipt_is_unused = receipt_candidate.as_ref().is_some_and(|receipt| {
            !memory.records.iter().any(|record| {
                record.verification_receipt_id.as_deref() == Some(receipt.verification_id())
            })
        });
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
        record.claim_status = if proposal_snapshot.claim_status == MemoryClaimStatus::UnverifiedTaskOutcome
            && receipt_is_unused
        {
            record.verification_receipt_id = receipt_candidate.as_ref().map(|receipt| receipt.verification_id().to_owned());
            record.evidence_hash = receipt_candidate.as_ref().map(|receipt| receipt.evidence_hash().to_owned());
            MemoryClaimStatus::VerifiedTaskOutcome
        } else {
            proposal_snapshot.claim_status
        };
        Ok(record_id)
    })?;

    proposal.status = ProposalStatus::Approved;
    proposal.decided_at_unix = Some(now);
    proposal.memory_record_id = Some(memory_record_id.clone());
    proposal.content.clear();
    proposal.tags.clear();
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
    proposal.content.clear();
    proposal.tags.clear();
    save_unlocked(directory, &store)
}


const RECEIPT_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;
const TRUSTED_VERIFIER_ID: &str = "rocksoul.core.verifier";
const TRUSTED_VERIFIER_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ReceiptRevocation {
    verification_id: String,
    revoked_at_unix: u64,
    reason_code: String,
}

pub fn revoke_receipt(
    receipts_directory: &Path,
    verification_id: &str,
    now: u64,
    reason_code: &str,
) -> Result<()> {
    if verification_id.trim().is_empty()
        || now == 0
        || reason_code.trim().is_empty()
        || reason_code.chars().any(char::is_control)
    {
        bail!("receipt revocation requires an ID, timestamp, and reason code");
    }
    fs::create_dir_all(receipts_directory)?;
    let path = receipts_directory.join("revoked.jsonl");
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    serde_json::to_writer(&mut file, &ReceiptRevocation {
        verification_id: verification_id.to_owned(),
        revoked_at_unix: now,
        reason_code: reason_code.to_owned(),
    })?;
    file.write_all(&[10])?;
    file.sync_all()?;
    Ok(())
}

fn is_receipt_revoked(receipts_directory: &Path, verification_id: &str) -> Result<bool> {
    let path = receipts_directory.join("revoked.jsonl");
    if !path.is_file() {
        return Ok(false);
    }
    let content = fs::read_to_string(&path)?;
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event: ReceiptRevocation = serde_json::from_str(line)
            .with_context(|| format!("receipt revocation ledger has invalid line {}", index + 1))?;
        if event.verification_id.trim().is_empty()
            || event.revoked_at_unix == 0
            || event.reason_code.trim().is_empty()
        {
            bail!("receipt revocation ledger contains an invalid event");
        }
        if event.verification_id == verification_id {
            return Ok(true);
        }
    }
    Ok(false)
}

fn load_verified_receipt(
    receipts_directory: &Path,
    verification_id: &str,
    claim: &str,
    now: u64,
) -> Result<Option<VerificationReceipt>> {
    if is_receipt_revoked(receipts_directory, verification_id)? {
        return Ok(None);
    }
    let expected_claim = normalize_task_claim(claim);
    let expected_claim_hash = format!("{:x}", Sha256::digest(expected_claim.as_bytes()));
    for entry in fs::read_dir(receipts_directory)? {
        let path = entry?.path();
        if !path.is_file()
            || path.file_name().is_some_and(|name| name == "revoked.jsonl")
        {
            continue;
        }
        let bytes = fs::read(&path)?;
        let document: serde_json::Value = serde_json::from_slice(&bytes)
            .with_context(|| format!("invalid verification receipt {}", path.display()))?;
        if document.get("verification_id").and_then(serde_json::Value::as_str)
            != Some(verification_id)
        {
            continue;
        }
        if document.get("schema_version").and_then(serde_json::Value::as_u64) != Some(2) {
            return Ok(None);
        }
        let receipt: VerificationReceipt = serde_json::from_value(document)?;
        if !receipt.integrity_valid()
            || receipt.outcome() != "verified"
            || receipt.verifier_id() != TRUSTED_VERIFIER_ID
            || receipt.verifier_version() != TRUSTED_VERIFIER_VERSION
            || receipt.normalized_claim() != Some(expected_claim.as_str())
            || receipt.claim_hash() != Some(expected_claim_hash.as_str())
            || receipt.evidence_hash().len() != 64
            || receipt.required_checks_passed() == 0
            || receipt.total_checks() < receipt.required_checks_passed()
            || receipt.verified_at_unix() > now
            || now.saturating_sub(receipt.verified_at_unix()) > RECEIPT_MAX_AGE_SECONDS
        {
            return Ok(None);
        }
        return Ok(Some(receipt));
    }
    Ok(None)
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
    if let Some(reason) = sensitive_content_reason(content)
        .or_else(|| sensitive_content_reason(source_turn_reference))
        .or_else(|| sensitive_content_reason(provenance))
        .or_else(|| tags.iter().find_map(|tag| sensitive_content_reason(tag)))
    {
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
        proposal.claim_status,
        &proposal.verification_receipt_id,
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
    let backup = directory.join(format!("{PROPOSAL_FILE}.bak"));
    if backup.exists() {
        fs::remove_file(&backup).with_context(|| {
            format!(
                "could not remove stale proposal backup {}",
                backup.display()
            )
        })?;
    }
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
    if backup.exists() {
        fs::remove_file(&backup).with_context(|| {
            format!("could not scrub stale proposal backup {}", backup.display())
        })?;
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
        MemoryProposalCandidate, ProposalStatus, approve, approve_with_receipts, cancel, create,
        deny, get_pending, list,
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
            claim_status: crate::memory::MemoryClaimStatus::Inference,
            verification_receipt_id: None,
        }
    }

    fn issue_receipt(
        receipts_directory: &std::path::Path,
        verification_id: &str,
        claim: &str,
        timestamp: u64,
    ) {
        use crate::core::{
            TurnEvent, TurnState, VerificationEvidence, VerificationOutcome,
            VerificationRequest, verify_turn_and_award_xp,
        };
        use crate::rid::{RuhIdentity, RuhProfile};

        let mut profile = RuhProfile::new(RuhIdentity {
            instance_id: "receipt-fixture".to_owned(),
            created_at_unix: 1,
        }).unwrap();
        let mut turn = TurnState::new("receipt-task", 0);
        turn.apply(TurnEvent::Start).unwrap();
        turn.apply(TurnEvent::PlanAccepted).unwrap();
        turn.apply(TurnEvent::ActionCompleted).unwrap();
        let request = VerificationRequest::new(
            "receipt-task",
            verification_id,
            timestamp,
            1,
            1,
            vec![VerificationEvidence::new(
                "artifact-readback",
                "present",
                "present",
                "test-evidence:artifact",
                true,
            )],
        ).with_claim(claim);
        let receipt = match verify_turn_and_award_xp(&mut turn, &mut profile, request).unwrap() {
            VerificationOutcome::Verified { receipt, .. } => receipt,
            other => panic!("expected a verified fixture receipt, got {other:?}"),
        };
        fs::create_dir_all(receipts_directory).unwrap();
        let mut document = serde_json::to_value(&receipt).unwrap();
        document.as_object_mut().unwrap().insert("schema_version".into(), serde_json::json!(2));
        document.as_object_mut().unwrap().insert("operation_id".into(), serde_json::json!("test-operation"));
        fs::write(
            receipts_directory.join("fixture-receipt.json"),
            serde_json::to_vec_pretty(&document).unwrap(),
        ).unwrap();
    }

    fn task_candidate(content: &str, receipt_id: Option<&str>) -> MemoryProposalCandidate {
        let mut value = candidate(
            crate::memory::MemoryKind::Episodic,
            content,
            "turn-task-outcome",
        );
        value.claim_status = crate::memory::MemoryClaimStatus::UnverifiedTaskOutcome;
        value.verification_receipt_id = receipt_id.map(str::to_owned);
        value
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
        let audit = list(&directory).unwrap();
        assert!(
            audit[0].content.is_empty(),
            "approved proposal payload must be scrubbed"
        );
        assert!(audit[0].tags.is_empty());
        assert!(
            !directory.join("proposals.v1.json.bak").exists(),
            "old proposal backup must not retain the approved candidate payload"
        );
        assert!(approve(&directory, &proposal.id, &proposal.approval_phrase(), 102).is_err());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn exact_fresh_kernel_receipt_is_required_for_verified_task_outcome() {
        let directory = temp_dir();
        let receipts = directory.join("receipts");
        let claim = "Created workspace artifact note.txt sha256 abcdef";
        issue_receipt(&receipts, "verify-task-exact", claim, 10_000);

        let proposal = create(
            &directory,
            task_candidate(claim, Some("verify-task-exact")),
            10_001,
        ).unwrap();
        let memory_id = approve_with_receipts(
            &directory,
            Some(&receipts),
            &proposal.id,
            &proposal.approval_phrase(),
            10_002,
        ).unwrap();
        let memory = crate::memory::MemoryStore::load(&directory).unwrap();
        let record = memory.records.iter().find(|record| record.id == memory_id).unwrap();
        assert_eq!(record.claim_status, crate::memory::MemoryClaimStatus::VerifiedTaskOutcome);
        assert_eq!(record.verification_receipt_id.as_deref(), Some("verify-task-exact"));
        assert!(record.evidence_hash.as_deref().is_some_and(|hash| hash.len() == 64));
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn missing_mismatched_stale_and_revoked_receipts_remain_unverified() {
        let directory = temp_dir();
        let receipts = directory.join("receipts");
        let claim = "The deployment completed successfully";

        let missing = create(&directory, task_candidate(claim, None), 20_000).unwrap();
        let missing_id = approve_with_receipts(
            &directory, Some(&receipts), &missing.id, &missing.approval_phrase(), 20_001,
        ).unwrap();
        let store = crate::memory::MemoryStore::load(&directory).unwrap();
        assert_eq!(store.records.iter().find(|r| r.id == missing_id).unwrap().claim_status,
            crate::memory::MemoryClaimStatus::UnverifiedTaskOutcome);

        issue_receipt(&receipts, "verify-mismatch", "different exact claim", 20_000);
        let mismatch = create(&directory, task_candidate(claim, Some("verify-mismatch")), 20_002).unwrap();
        let mismatch_id = approve_with_receipts(
            &directory, Some(&receipts), &mismatch.id, &mismatch.approval_phrase(), 20_003,
        ).unwrap();
        let store = crate::memory::MemoryStore::load(&directory).unwrap();
        assert_eq!(store.records.iter().find(|r| r.id == mismatch_id).unwrap().claim_status,
            crate::memory::MemoryClaimStatus::UnverifiedTaskOutcome);

        issue_receipt(&receipts, "verify-stale", claim, 1);
        let stale = create(&directory, task_candidate(claim, Some("verify-stale")), 1 + 8 * 24 * 60 * 60).unwrap();
        let stale_id = approve_with_receipts(
            &directory, Some(&receipts), &stale.id, &stale.approval_phrase(), 1 + 8 * 24 * 60 * 60,
        ).unwrap();
        let store = crate::memory::MemoryStore::load(&directory).unwrap();
        assert_eq!(store.records.iter().find(|r| r.id == stale_id).unwrap().claim_status,
            crate::memory::MemoryClaimStatus::UnverifiedTaskOutcome);

        issue_receipt(&receipts, "verify-revoked", claim, 30_000);
        super::revoke_receipt(&receipts, "verify-revoked", 30_001, "operator_review").unwrap();
        let revoked = create(&directory, task_candidate(claim, Some("verify-revoked")), 30_002).unwrap();
        let revoked_id = approve_with_receipts(
            &directory, Some(&receipts), &revoked.id, &revoked.approval_phrase(), 30_003,
        ).unwrap();
        let store = crate::memory::MemoryStore::load(&directory).unwrap();
        assert_eq!(store.records.iter().find(|r| r.id == revoked_id).unwrap().claim_status,
            crate::memory::MemoryClaimStatus::UnverifiedTaskOutcome);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn receipt_is_one_time_and_cannot_verify_a_second_memory_claim() {
        let directory = temp_dir();
        let receipts = directory.join("receipts");
        let claim = "Task completed with verified evidence";
        issue_receipt(&receipts, "verify-one-time", claim, 40_000);

        let first = create(&directory, task_candidate(claim, Some("verify-one-time")), 40_001).unwrap();
        let first_id = approve_with_receipts(
            &directory, Some(&receipts), &first.id, &first.approval_phrase(), 40_002,
        ).unwrap();
        let second = create(&directory, task_candidate(claim, Some("verify-one-time")), 40_003).unwrap();
        let second_id = approve_with_receipts(
            &directory, Some(&receipts), &second.id, &second.approval_phrase(), 40_004,
        ).unwrap();

        let store = crate::memory::MemoryStore::load(&directory).unwrap();
        assert_eq!(store.records.iter().find(|r| r.id == first_id).unwrap().claim_status,
            crate::memory::MemoryClaimStatus::VerifiedTaskOutcome);
        assert_eq!(store.records.iter().find(|r| r.id == second_id).unwrap().claim_status,
            crate::memory::MemoryClaimStatus::UnverifiedTaskOutcome);
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
        let audit = list(&directory).unwrap();
        assert_eq!(
            audit
                .iter()
                .filter(|item| item.status == ProposalStatus::Denied)
                .count(),
            1
        );
        assert!(audit.iter().all(|item| item.content.is_empty()));
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
                    candidate(crate::memory::MemoryKind::Semantic, content, "turn-secret",),
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
