use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow, bail};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub const MEMORY_SCHEMA_VERSION: u16 = 1;
const STORE_FILE: &str = "store.v1.json";

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
        let path = store_path(directory);
        if path.exists() || backup_path(&path).exists() {
            return Ok((Self::load(directory)?, false));
        }
        let store = Self::default();
        store.save(directory)?;
        Ok((store, true))
    }

    pub fn load(directory: &Path) -> Result<Self> {
        let path = store_path(directory);
        if !path.exists() {
            let backup = backup_path(&path);
            if backup.is_file() {
                fs::rename(&backup, &path).with_context(|| {
                    format!("could not recover memory backup {}", backup.display())
                })?;
            } else {
                return Ok(Self::default());
            }
        }

        let bytes = fs::read(&path)
            .with_context(|| format!("could not read memory store {}", path.display()))?;
        let store: Self = serde_json::from_slice(&bytes)
            .with_context(|| format!("memory store is not valid JSON: {}", path.display()))?;
        store.validate()?;
        Ok(store)
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

    pub fn save(&self, directory: &Path) -> Result<()> {
        self.validate()?;
        fs::create_dir_all(directory).with_context(|| {
            format!("could not create memory directory {}", directory.display())
        })?;
        let path = store_path(directory);
        write_json_atomic(&path, self)
    }

    pub fn export(&self, directory: &Path) -> Result<PathBuf> {
        self.validate()?;
        let export_dir = directory.join("exports");
        fs::create_dir_all(&export_dir).with_context(|| {
            format!("could not create export directory {}", export_dir.display())
        })?;
        let path = export_dir.join("latest.json");
        let export = MemoryExport {
            schema_version: self.schema_version,
            revision: self.revision,
            exported_at_unix: now_unix()?,
            records: self.records.clone(),
        };
        write_json_atomic(&path, &export)?;
        Ok(path)
    }
}

#[derive(Serialize)]
struct MemoryExport {
    schema_version: u16,
    revision: u64,
    exported_at_unix: u64,
    records: Vec<MemoryRecord>,
}

pub fn now_unix() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_secs())
}

fn store_path(directory: &Path) -> PathBuf {
    directory.join(STORE_FILE)
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
                .with_context(|| format!("could not remove stale backup {}", backup.display()))?;
        }
        fs::rename(path, &backup)
            .with_context(|| format!("could not stage previous memory file {}", path.display()))?;
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

    if moved_previous {
        let _ = fs::remove_file(backup);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{MEMORY_SCHEMA_VERSION, MemoryKind, MemoryStore};

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("rocksoul-memory-{}-{nanos}", std::process::id()))
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
        let directory = temp_dir();
        let mut store = MemoryStore::default();
        store.schema_version = MEMORY_SCHEMA_VERSION + 1;
        assert!(store.validate().is_err());
        store.schema_version = MEMORY_SCHEMA_VERSION;
        store.records.push(super::MemoryRecord {
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
        });
        assert!(store.validate().is_err());
        let _ = fs::remove_dir_all(directory);
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
