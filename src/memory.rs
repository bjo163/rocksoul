use std::{
    collections::HashSet,
    fs,
    fs::{File, OpenOptions},
    io::ErrorKind,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use fs2::FileExt;

use anyhow::{Context, Result, anyhow, bail};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub const MEMORY_SCHEMA_VERSION: u16 = 3;
pub const RETENTION_POLICY_VERSION: u16 = 1;
const STORE_FILE: &str = "store.v3.json";
const LEGACY_V2_STORE_FILE: &str = "store.v2.json";
const LEGACY_STORE_FILE: &str = "store.v1.json";
const FORGETTING_LEDGER_FILE: &str = "forgetting.jsonl";
const LOCK_FILE: &str = ".memory.lock";
const LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(25);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    Episodic,
    Semantic,
    Procedural,
    Working,
    Derived,
}

impl std::fmt::Display for MemoryKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Episodic => "episodic",
            Self::Semantic => "semantic",
            Self::Procedural => "procedural",
            Self::Working => "working",
            Self::Derived => "derived",
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionPolicy {
    #[default]
    KeepUntilDeleted,
    ExpireAt,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MemoryRecord {
    pub id: String,
    pub kind: MemoryKind,
    pub content: String,
    pub tags: Vec<String>,
    pub source: String,
    pub user_approved: bool,
    pub confidence: Option<f32>,
    pub created_at_unix: u64,
    pub updated_at_unix: u64,
    pub expires_at_unix: Option<u64>,
    #[serde(default = "default_retention_policy_version")]
    pub retention_policy_version: u16,
    #[serde(default)]
    pub retention_policy: RetentionPolicy,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MemoryStore {
    pub schema_version: u16,
    pub revision: u64,
    pub records: Vec<MemoryRecord>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self {
            schema_version: MEMORY_SCHEMA_VERSION,
            revision: 0,
            records: Vec::new(),
        }
    }
}

impl MemoryStore {
    pub fn initialize(directory: &Path) -> Result<(Self, bool)> {
        fs::create_dir_all(directory).with_context(|| {
            format!("could not create memory directory {}", directory.display())
        })?;
        let _lock = acquire_store_lock(directory)?;
        let path = store_path(directory);
        let legacy = legacy_store_path(directory);
        if path.exists()
            || backup_path(&path).exists()
            || legacy.exists()
            || legacy_v2_store_path(directory).exists()
        {
            return Ok((Self::load_unlocked(directory)?, false));
        }
        let store = Self::default();
        store.save_unlocked(directory)?;
        Ok((store, true))
    }

    /// Load the current store, recover its last known-good backup when needed,
    /// or migrate the legacy v1 document without modifying the original file.
    pub fn load(directory: &Path) -> Result<Self> {
        fs::create_dir_all(directory).with_context(|| {
            format!("could not create memory directory {}", directory.display())
        })?;
        let _lock = acquire_store_lock(directory)?;
        Self::load_unlocked(directory)
    }

    fn load_unlocked(directory: &Path) -> Result<Self> {
        let path = store_path(directory);
        let backup = backup_path(&path);
        if path.exists() {
            match read_current_store(&path) {
                Ok(mut store) => {
                    apply_forgetting_ledger(directory, &mut store)?;
                    return Ok(store);
                },
                Err(primary_error) => {
                    // Unknown future versions must fail closed; never replace them
                    // with an older backup or legacy document.
                    if is_unsupported_future_version(&path)? {
                        return Err(primary_error);
                    }
                    if backup.is_file() {
                        let recovered = read_current_store(&backup).with_context(|| {
                            format!(
                                "primary memory store failed and backup {} is also invalid",
                                backup.display()
                            )
                        })?;
                        restore_backup(&backup, &path)?;
                        let mut recovered = recovered;
                        apply_forgetting_ledger(directory, &mut recovered)?;
                        scrub_forgotten_snapshots(directory)?;
                        eprintln!(
                            "Recovered memory store from validated backup: {}",
                            backup.display()
                        );
                        return Ok(recovered);
                    }
                    return Err(primary_error).context("no valid memory backup is available");
                }
            }
        }

        if backup.is_file() {
            let mut recovered = read_current_store(&backup)?;
            restore_backup(&backup, &path)?;
            apply_forgetting_ledger(directory, &mut recovered)?;
            scrub_forgotten_snapshots(directory)?;
            eprintln!(
                "Recovered missing memory store from validated backup: {}",
                backup.display()
            );
            return Ok(recovered);
        }

        for (legacy, expected_version) in [
            (legacy_v2_store_path(directory), 2_u16),
            (legacy_store_path(directory), 1_u16),
        ] {
            if !legacy.is_file() {
                continue;
            }
            let bytes = fs::read(&legacy).with_context(|| {
                format!("could not read legacy memory store {}", legacy.display())
            })?;
            let mut store: Self = serde_json::from_slice(&bytes).with_context(|| {
                format!("legacy memory store is not valid JSON: {}", legacy.display())
            })?;
            if store.schema_version != expected_version {
                bail!(
                    "legacy memory store {} has unsupported schema version {}; expected {}",
                    legacy.display(),
                    store.schema_version,
                    expected_version
                );
            }
            store.schema_version = MEMORY_SCHEMA_VERSION;
            for record in &mut store.records {
                record.retention_policy_version = RETENTION_POLICY_VERSION;
                record.retention_policy = if record.expires_at_unix.is_some() {
                    RetentionPolicy::ExpireAt
                } else {
                    RetentionPolicy::KeepUntilDeleted
                };
            }
            store.validate()?;
            apply_forgetting_ledger(directory, &mut store)?;
            store.save_unlocked(directory)?;
            eprintln!(
                "Migrated memory store schema v{} to v{}; original retained at {}",
                expected_version,
                MEMORY_SCHEMA_VERSION,
                legacy.display()
            );
            return Ok(store);
        }

        Ok(Self::default())
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != MEMORY_SCHEMA_VERSION {
            bail!(
                "unsupported memory schema version {}; this runtime supports {}",
                self.schema_version,
                MEMORY_SCHEMA_VERSION
            );
        }

        let mut ids = HashSet::with_capacity(self.records.len());
        for record in &self.records {
            if record.id.trim().is_empty()
                || record.content.trim().is_empty()
                || record.source.trim().is_empty()
                || record.created_at_unix == 0
                || record.updated_at_unix < record.created_at_unix
                || !ids.insert(record.id.as_str())
            {
                bail!("memory store contains an invalid or duplicate record");
            }
            if record
                .confidence
                .is_some_and(|confidence| !(0.0..=1.0).contains(&confidence))
            {
                bail!("memory confidence must be between 0.0 and 1.0");
            }
            if record
                .expires_at_unix
                .is_some_and(|expires| expires <= record.created_at_unix)
            {
                bail!("memory expiry must be later than creation time");
            }
            if record.tags.iter().any(|tag| tag.trim().is_empty()) {
                bail!("memory tags must not be empty");
            }
            if record.retention_policy_version != RETENTION_POLICY_VERSION {
                bail!("unsupported memory retention policy version {}", record.retention_policy_version);
            }
            if (record.retention_policy == RetentionPolicy::ExpireAt)
                != record.expires_at_unix.is_some()
            {
                bail!("memory retention policy and expiry timestamp are inconsistent");
            }
        }
        Ok(())
    }

    pub fn add_user_entry(
        &mut self,
        kind: MemoryKind,
        content: String,
        tags: Vec<String>,
        expires_at_unix: Option<u64>,
        now: u64,
    ) -> Result<String> {
        if content.trim().is_empty() || now == 0 {
            bail!("memory content and timestamp are required");
        }
        if expires_at_unix.is_some_and(|expires| expires <= now) {
            bail!("memory expiry must be in the future");
        }

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock is before the Unix epoch")?
            .as_nanos();
        let id = format!("mem-{now}-{nanos}-{}", std::process::id());
        let record = MemoryRecord {
            id: id.clone(),
            kind,
            content: content.trim().to_owned(),
            tags: normalize_tags(tags),
            source: "user_explicit".to_owned(),
            user_approved: true,
            confidence: None,
            created_at_unix: now,
            updated_at_unix: now,
            expires_at_unix,
            retention_policy_version: RETENTION_POLICY_VERSION,
            retention_policy: if expires_at_unix.is_some() {
                RetentionPolicy::ExpireAt
            } else {
                RetentionPolicy::KeepUntilDeleted
            },
        };
        self.records.push(record);
        self.revision = self.revision.saturating_add(1);
        self.validate()?;
        Ok(id)
    }

    pub fn active_records(&self, now: u64) -> Vec<&MemoryRecord> {
        self.records
            .iter()
            .filter(|record| record.expires_at_unix.is_none_or(|expires| expires > now))
            .collect()
    }

    pub fn search(&self, query: &str, now: u64) -> Vec<&MemoryRecord> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return Vec::new();
        }
        self.active_records(now)
            .into_iter()
            .filter(|record| {
                record.content.to_lowercase().contains(&needle)
                    || record
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&needle))
            })
            .collect()
    }

    pub fn delete(&mut self, id: &str) -> bool {
        let previous = self.records.len();
        self.records.retain(|record| record.id != id);
        let deleted = self.records.len() != previous;
        if deleted {
            self.revision = self.revision.saturating_add(1);
        }
        deleted
    }

    pub fn clear(&mut self) {
        if !self.records.is_empty() {
            self.records.clear();
            self.revision = self.revision.saturating_add(1);
        }
    }

    #[cfg(test)]
    fn save(&self, directory: &Path) -> Result<()> {
        self.validate()?;
        fs::create_dir_all(directory).with_context(|| {
            format!("could not create memory directory {}", directory.display())
        })?;
        let _lock = acquire_store_lock(directory)?;
        self.save_unlocked(directory)
    }

    fn save_unlocked(&self, directory: &Path) -> Result<()> {
        self.validate()?;
        write_json_atomic(&store_path(directory), self)
    }

    pub fn transact<T>(
        directory: &Path,
        operation: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        fs::create_dir_all(directory).with_context(|| {
            format!("could not create memory directory {}", directory.display())
        })?;
        let _lock = acquire_store_lock(directory)?;
        let path = store_path(directory);
        let existed = path.is_file()
            || backup_path(&path).is_file()
            || legacy_store_path(directory).is_file();
        let mut store = Self::load_unlocked(directory)?;
        let previous = store.clone();
        let output = operation(&mut store)?;
        let remaining = store.records.iter().map(|record| record.id.as_str()).collect::<HashSet<_>>();
        let removed = previous.records.iter()
            .filter(|record| !remaining.contains(record.id.as_str()))
            .collect::<Vec<_>>();
        if !removed.is_empty() {
            let now = now_unix()?;
            let events = removed.iter().map(|record| ForgetEvent {
                id: record.id.clone(),
                forgotten_at_unix: now,
                reason: if record.expires_at_unix.is_some_and(|expires| expires <= now) {
                    "retention_expired".to_owned()
                } else {
                    "user_forget".to_owned()
                },
            }).collect::<Vec<_>>();
            append_forgetting_events(directory, &events)?;
        }
        if !existed || store != previous {
            store.save_unlocked(directory)?;
        }
        if !removed.is_empty() {
            scrub_forgotten_snapshots(directory)?;
        }
        Ok(output)
    }

    pub fn export(&self, directory: &Path) -> Result<PathBuf> {
        self.validate()?;
        fs::create_dir_all(directory).with_context(|| {
            format!("could not create memory directory {}", directory.display())
        })?;
        let _lock = acquire_store_lock(directory)?;
        self.export_unlocked(directory)
    }

    fn export_unlocked(&self, directory: &Path) -> Result<PathBuf> {
        self.validate()?;
        let export_dir = directory.join("exports");
        fs::create_dir_all(&export_dir).with_context(|| {
            format!("could not create export directory {}", export_dir.display())
        })?;
        let path = export_dir.join("latest.json");
        let now = now_unix()?;
        let forgotten_ids = load_forgetting_ids(directory)?.into_iter().collect();
        let export = MemoryExport {
            schema_version: self.schema_version,
            retention_policy_version: RETENTION_POLICY_VERSION,
            revision: self.revision,
            exported_at_unix: now,
            forgotten_ids,
            records: self.active_records(now).into_iter().cloned().collect(),
        };
        write_json_atomic(&path, &export)?;
        Ok(path)
    }
}

#[derive(Serialize)]
struct MemoryExport {
    schema_version: u16,
    retention_policy_version: u16,
    revision: u64,
    exported_at_unix: u64,
    forgotten_ids: Vec<String>,
    records: Vec<MemoryRecord>,
}

pub fn now_unix() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_secs())
}

fn acquire_store_lock(directory: &Path) -> Result<File> {
    fs::create_dir_all(directory)?;
    let lock_path = directory.join(LOCK_FILE);
    let started = Instant::now();
    let file = loop {
        match OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(file) => break file,
            Err(error) if is_lock_contention(&error) => {
                if started.elapsed() >= LOCK_TIMEOUT {
                    bail!(
                        "memory store is busy; could not open lock {} within {} seconds. Close other Rocksoul operations and retry",
                        lock_path.display(),
                        LOCK_TIMEOUT.as_secs()
                    );
                }
                thread::sleep(LOCK_RETRY_INTERVAL);
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("could not open memory lock {}", lock_path.display())
                });
            }
        }
    };

    loop {
        match file.try_lock_exclusive() {
            Ok(()) => return Ok(file),
            Err(error) if is_lock_contention(&error) => {
                if started.elapsed() >= LOCK_TIMEOUT {
                    bail!(
                        "memory store is busy; could not acquire {} within {} seconds. Close other Rocksoul operations and retry",
                        lock_path.display(),
                        LOCK_TIMEOUT.as_secs()
                    );
                }
                thread::sleep(LOCK_RETRY_INTERVAL);
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("could not lock memory store {}", lock_path.display())
                });
            }
        }
    }
}

fn is_lock_contention(error: &std::io::Error) -> bool {
    if error.kind() == ErrorKind::WouldBlock {
        return true;
    }

    #[cfg(windows)]
    {
        // Windows reports LockFileEx contention as ERROR_LOCK_VIOLATION or
        // ERROR_SHARING_VIOLATION, which Rust may classify as PermissionDenied.
        matches!(error.raw_os_error(), Some(32 | 33))
    }

    #[cfg(not(windows))]
    {
        false
    }
}

fn store_path(directory: &Path) -> PathBuf {
    directory.join(STORE_FILE)
}

fn legacy_store_path(directory: &Path) -> PathBuf {
    directory.join(LEGACY_STORE_FILE)
}

fn legacy_v2_store_path(directory: &Path) -> PathBuf {
    directory.join(LEGACY_V2_STORE_FILE)
}

fn default_retention_policy_version() -> u16 {
    RETENTION_POLICY_VERSION
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ForgetEvent {
    id: String,
    forgotten_at_unix: u64,
    reason: String,
}

fn forgetting_ledger_path(directory: &Path) -> PathBuf {
    directory.join(FORGETTING_LEDGER_FILE)
}

fn load_forgetting_ids(directory: &Path) -> Result<HashSet<String>> {
    let path = forgetting_ledger_path(directory);
    if !path.is_file() {
        return Ok(HashSet::new());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("could not read forgetting ledger {}", path.display()))?;
    let mut ids = HashSet::new();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event: ForgetEvent = serde_json::from_str(line).with_context(|| {
            format!(
                "forgetting ledger {} has invalid event on line {}",
                path.display(),
                index + 1
            )
        })?;
        if event.id.trim().is_empty()
            || event.forgotten_at_unix == 0
            || event.reason.trim().is_empty()
        {
            bail!("forgetting ledger contains an invalid event on line {}", index + 1);
        }
        ids.insert(event.id);
    }
    Ok(ids)
}

fn append_forgetting_events(directory: &Path, events: &[ForgetEvent]) -> Result<()> {
    if events.is_empty() {
        return Ok(());
    }
    let path = forgetting_ledger_path(directory);
    let mut bytes = Vec::new();
    for event in events {
        serde_json::to_writer(&mut bytes, event)?;
        bytes.push(10);
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("could not append forgetting ledger {}", path.display()))?;
    use std::io::Write;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

fn apply_forgetting_ledger(directory: &Path, store: &mut MemoryStore) -> Result<()> {
    let forgotten = load_forgetting_ids(directory)?;
    let before = store.records.len();
    store.records.retain(|record| !forgotten.contains(&record.id));
    if store.records.len() != before {
        store.revision = store.revision.saturating_add(1);
        store.save_unlocked(directory)?;
        scrub_forgotten_snapshots(directory)?;
    }
    Ok(())
}

fn scrub_forgotten_snapshots(directory: &Path) -> Result<()> {
    let current = store_path(directory);
    for path in [
        backup_path(&current),
        legacy_store_path(directory),
        legacy_v2_store_path(directory),
        directory.join("exports/latest.json"),
    ] {
        if path.is_file() {
            fs::remove_file(&path).with_context(|| {
                format!("could not remove obsolete memory snapshot {}", path.display())
            })?;
        }
    }
    Ok(())
}

fn read_current_store(path: &Path) -> Result<MemoryStore> {
    let bytes = fs::read(path)
        .with_context(|| format!("could not read memory store {}", path.display()))?;
    let store: MemoryStore = serde_json::from_slice(&bytes)
        .with_context(|| format!("memory store is not valid JSON: {}", path.display()))?;
    store.validate()?;
    Ok(store)
}

fn is_unsupported_future_version(path: &Path) -> Result<bool> {
    let bytes = fs::read(path)?;
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return Ok(false);
    };
    Ok(value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|version| version > u64::from(MEMORY_SCHEMA_VERSION)))
}

fn restore_backup(backup: &Path, destination: &Path) -> Result<()> {
    let temporary = destination.with_extension("json.recovery.tmp");
    fs::copy(backup, &temporary)
        .with_context(|| format!("could not stage memory backup {}", backup.display()))?;
    if destination.exists() {
        fs::remove_file(destination).with_context(|| {
            format!(
                "could not remove corrupt memory store {}",
                destination.display()
            )
        })?;
    }
    fs::rename(&temporary, destination)
        .with_context(|| format!("could not restore memory backup {}", backup.display()))?;
    Ok(())
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.bak")
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut normalized = tags
        .into_iter()
        .map(|tag| tag.trim().to_lowercase())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("memory file path has no parent directory"))?;
    fs::create_dir_all(parent)?;
    let temporary = path.with_extension("json.tmp");
    let backup = backup_path(path);
    let bytes = serde_json::to_vec_pretty(value).context("could not serialize memory data")?;
    fs::write(&temporary, bytes).with_context(|| {
        format!(
            "could not write temporary memory file {}",
            temporary.display()
        )
    })?;

    let moved_previous = if path.exists() {
        if backup.exists() {
            fs::remove_file(&backup)
                .with_context(|| format!("could not rotate memory backup {}", backup.display()))?;
        }
        fs::rename(path, &backup).with_context(|| {
            format!("could not preserve previous memory file {}", path.display())
        })?;
        true
    } else {
        false
    };

    if let Err(error) = fs::rename(&temporary, path) {
        if moved_previous {
            let _ = fs::rename(&backup, path);
        }
        let _ = fs::remove_file(&temporary);
        return Err(error)
            .with_context(|| format!("could not replace memory file {}", path.display()));
    }

    // Keep the previous valid generation as a durable recovery point.
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    use fs2::FileExt;

    use super::{
        MEMORY_SCHEMA_VERSION, MemoryKind, MemoryStore, backup_path, legacy_store_path, store_path,
    };

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("rocksoul-memory-{}-{nanos}", std::process::id()))
    }

    #[test]
    fn migrates_v1_store_to_v2_without_modifying_legacy_file() {
        let directory = temp_dir();
        fs::create_dir_all(&directory).unwrap();
        let mut legacy = MemoryStore {
            schema_version: 1,
            revision: 7,
            ..MemoryStore::default()
        };
        legacy.records.push(super::MemoryRecord {
            id: "legacy-id".into(),
            kind: MemoryKind::Semantic,
            content: "preserve this record".into(),
            tags: vec!["legacy".into()],
            source: "user_explicit".into(),
            user_approved: true,
            confidence: Some(0.75),
            created_at_unix: 10,
            updated_at_unix: 12,
            expires_at_unix: Some(100),
        });
        let legacy_path = legacy_store_path(&directory);
        fs::write(&legacy_path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();

        let migrated = MemoryStore::load(&directory).unwrap();
        assert_eq!(migrated.schema_version, MEMORY_SCHEMA_VERSION);
        assert_eq!(migrated.revision, 7);
        assert_eq!(migrated.records, legacy.records);
        assert!(legacy_path.is_file());
        assert!(store_path(&directory).is_file());
        assert_eq!(MemoryStore::load(&directory).unwrap(), migrated);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn keeps_previous_valid_generation_as_recovery_backup() {
        let directory = temp_dir();
        let (mut store, _) = MemoryStore::initialize(&directory).unwrap();
        store
            .add_user_entry(MemoryKind::Semantic, "first".into(), vec![], None, 10)
            .unwrap();
        store.save(&directory).unwrap();
        let backup = backup_path(&store_path(&directory));
        assert!(backup.is_file());
        let previous: MemoryStore = serde_json::from_slice(&fs::read(&backup).unwrap()).unwrap();
        assert!(previous.records.is_empty());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn recovers_corrupt_current_store_from_valid_backup() {
        let directory = temp_dir();
        let (mut store, _) = MemoryStore::initialize(&directory).unwrap();
        store
            .add_user_entry(MemoryKind::Semantic, "recoverable".into(), vec![], None, 10)
            .unwrap();
        store.save(&directory).unwrap();
        let path = store_path(&directory);
        let backup = backup_path(&path);
        let previous: MemoryStore = serde_json::from_slice(&fs::read(&backup).unwrap()).unwrap();
        fs::write(&path, b"{ truncated").unwrap();

        let recovered = MemoryStore::load(&directory).unwrap();
        assert_eq!(recovered, previous);
        let on_disk = MemoryStore::load(&directory).unwrap();
        assert_eq!(on_disk, previous);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn transaction_persists_only_committed_revision_changes() {
        let directory = temp_dir();
        MemoryStore::transact(&directory, |store| {
            store.add_user_entry(
                MemoryKind::Semantic,
                "committed under lock".into(),
                vec![],
                None,
                20,
            )?;
            Ok(())
        })
        .unwrap();

        let loaded = MemoryStore::load(&directory).unwrap();
        assert_eq!(loaded.revision, 1);
        assert_eq!(loaded.records.len(), 1);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn cross_process_lock_contention_has_bounded_timeout() {
        if let Ok(directory) = std::env::var("ROCKSOUL_LOCK_CHILD_DIR") {
            let directory = PathBuf::from(directory);
            if std::env::var_os("ROCKSOUL_LOCK_CHILD_WRITE").is_some() {
                let content = std::env::var("ROCKSOUL_LOCK_CHILD_WRITE").unwrap();
                MemoryStore::transact(&directory, |store| {
                    store.add_user_entry(MemoryKind::Semantic, content, vec![], None, 30)?;
                    Ok(())
                })
                .unwrap();
                return;
            }
            if std::env::var_os("ROCKSOUL_LOCK_CHILD_EXIT").is_some() {
                let _lock = super::acquire_store_lock(&directory).unwrap();
                std::process::exit(0);
            }
            let error = MemoryStore::load(&directory).unwrap_err();
            assert!(
                format!("{error:#}").contains("memory store is busy"),
                "unexpected lock contention result: {error:#}"
            );
            return;
        }

        let directory = temp_dir();
        fs::create_dir_all(&directory).unwrap();
        let lock_path = directory.join(super::LOCK_FILE);
        let held = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .unwrap();
        held.try_lock_exclusive().unwrap();

        let status = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("memory::tests::cross_process_lock_contention_has_bounded_timeout")
            .arg("--nocapture")
            .env("ROCKSOUL_LOCK_CHILD_DIR", &directory)
            .status()
            .unwrap();
        assert!(
            status.success(),
            "child process should report bounded lock timeout"
        );
        FileExt::unlock(&held).unwrap();

        let exit_status = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("memory::tests::cross_process_lock_contention_has_bounded_timeout")
            .arg("--nocapture")
            .env("ROCKSOUL_LOCK_CHILD_DIR", &directory)
            .env("ROCKSOUL_LOCK_CHILD_EXIT", "1")
            .status()
            .unwrap();
        assert!(
            exit_status.success(),
            "child process should exit while holding the OS lock"
        );
        assert!(
            MemoryStore::load(&directory).is_ok(),
            "OS lock must be released when its process exits"
        );

        let executable = std::env::current_exe().unwrap();
        let child_a = Command::new(&executable)
            .arg("--exact")
            .arg("memory::tests::cross_process_lock_contention_has_bounded_timeout")
            .arg("--nocapture")
            .env("ROCKSOUL_LOCK_CHILD_DIR", &directory)
            .env("ROCKSOUL_LOCK_CHILD_WRITE", "writer A")
            .spawn()
            .unwrap();
        let child_b = Command::new(&executable)
            .arg("--exact")
            .arg("memory::tests::cross_process_lock_contention_has_bounded_timeout")
            .arg("--nocapture")
            .env("ROCKSOUL_LOCK_CHILD_DIR", &directory)
            .env("ROCKSOUL_LOCK_CHILD_WRITE", "writer B")
            .spawn()
            .unwrap();
        assert!(child_a.wait_with_output().unwrap().status.success());
        assert!(child_b.wait_with_output().unwrap().status.success());
        let combined = MemoryStore::load(&directory).unwrap();
        assert_eq!(
            combined.records.len(),
            2,
            "concurrent writers must not lose either committed record"
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn rejects_unknown_future_version_without_falling_back_to_backup() {
        let directory = temp_dir();
        let (store, _) = MemoryStore::initialize(&directory).unwrap();
        let path = store_path(&directory);
        let backup = backup_path(&path);
        fs::copy(&path, &backup).unwrap();
        let future = serde_json::json!({"schema_version": 99, "revision": 0, "records": []});
        fs::write(&path, serde_json::to_vec(&future).unwrap()).unwrap();
        assert!(MemoryStore::load(&directory).is_err());
        let still_future: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(still_future["schema_version"], 99);
        drop(store);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn initializes_once_and_persists_only_explicit_user_entries() {
        let directory = temp_dir();
        let (mut store, created) = MemoryStore::initialize(&directory).unwrap();
        assert!(created);
        let id = store
            .add_user_entry(
                MemoryKind::Semantic,
                "User prefers concise technical explanations".into(),
                vec!["Preference".into(), "technical".into()],
                None,
                1_800_000_000,
            )
            .unwrap();
        assert!(store.records[0].user_approved);
        assert_eq!(store.records[0].source, "user_explicit");
        store.save(&directory).unwrap();

        let (loaded, created_again) = MemoryStore::initialize(&directory).unwrap();
        assert!(!created_again);
        assert_eq!(loaded.schema_version, MEMORY_SCHEMA_VERSION);
        assert_eq!(loaded.records[0].id, id);
        assert_eq!(loaded.records[0].tags, vec!["preference", "technical"]);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn search_matches_content_and_tags_case_insensitively() {
        let directory = temp_dir();
        let (mut store, _) = MemoryStore::initialize(&directory).unwrap();
        store
            .add_user_entry(
                MemoryKind::Procedural,
                "Use cargo fmt before submitting Rust changes".into(),
                vec!["Rust".into()],
                None,
                1_800_000_000,
            )
            .unwrap();
        assert_eq!(store.search("CARGO", 1_800_000_001).len(), 1);
        assert_eq!(store.search("rust", 1_800_000_001).len(), 1);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn expired_working_memory_is_not_returned() {
        let directory = temp_dir();
        let (mut store, _) = MemoryStore::initialize(&directory).unwrap();
        store
            .add_user_entry(
                MemoryKind::Working,
                "Temporary task context".into(),
                vec![],
                Some(1_800_000_100),
                1_800_000_000,
            )
            .unwrap();
        assert_eq!(store.active_records(1_800_000_101).len(), 0);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn delete_and_clear_remove_records() {
        let directory = temp_dir();
        let (mut store, _) = MemoryStore::initialize(&directory).unwrap();
        let id = store
            .add_user_entry(
                MemoryKind::Episodic,
                "A user-approved event".into(),
                vec![],
                None,
                1_800_000_000,
            )
            .unwrap();
        assert!(store.delete(&id));
        assert!(!store.delete(&id));
        store.clear();
        assert!(store.records.is_empty());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn rejects_unknown_schema_and_invalid_confidence() {
        let unsupported = MemoryStore {
            schema_version: MEMORY_SCHEMA_VERSION + 1,
            ..MemoryStore::default()
        };
        assert!(unsupported.validate().is_err());

        let invalid_confidence = MemoryStore {
            records: vec![super::MemoryRecord {
                id: "invalid".into(),
                kind: MemoryKind::Derived,
                content: "derived claim".into(),
                tags: vec![],
                source: "model".into(),
                user_approved: false,
                confidence: Some(1.5),
                created_at_unix: 1,
                updated_at_unix: 1,
                expires_at_unix: None,
            }],
            ..MemoryStore::default()
        };
        assert!(invalid_confidence.validate().is_err());
    }

    #[test]
    fn export_is_written_under_the_memory_root() {
        let directory = temp_dir();
        let (store, _) = MemoryStore::initialize(&directory).unwrap();
        let exported = store.export(&directory).unwrap();
        assert_eq!(exported, directory.join("exports/latest.json"));
        assert!(exported.is_file());
        fs::remove_dir_all(directory).unwrap();
    }
}
