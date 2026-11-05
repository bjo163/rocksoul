use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::rid::{LegacyStatArchive, RuhIdentity, RuhProfile, XpLedger, STAT_COUNT};

#[derive(Deserialize)]
struct LegacyRuhProfile {
    schema_version: u16,
    level_curve_version: u16,
    identity: RuhIdentity,
    xp_ledger: XpLedger,
    stats: Vec<Value>,
}

/// Load and validate a local RID profile, applying only explicit schema migrations.
pub fn load(path: &Path) -> Result<RuhProfile> {
    if !path.exists() {
        let backup = backup_path(path);
        if backup.is_file() {
            fs::rename(&backup, path)
                .with_context(|| format!("could not recover RID backup {}", backup.display()))?;
        } else {
            return Err(anyhow!("RID profile does not exist at {}", path.display()));
        }
    }

    let bytes = fs::read(path).with_context(|| format!("could not read {}", path.display()))?;
    let value: Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("RID profile is not valid JSON: {}", path.display()))?;
    let schema_version = value
        .get("schema_version")
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("RID profile has no valid schema_version"))?;

    let profile = match schema_version {
        1 => migrate_v1_profile(path, &bytes, value)?,
        2 => serde_json::from_value::<RuhProfile>(value)
            .context("RID v2 profile could not be decoded")?,
        other => return Err(anyhow!("unsupported RID profile schema version: {other}")),
    };
    profile
        .validate()
        .map_err(|error| anyhow!("RID profile validation failed: {error}"))?;
    Ok(profile)
}

fn migrate_v1_profile(path: &Path, original_bytes: &[u8], value: Value) -> Result<RuhProfile> {
    let legacy: LegacyRuhProfile = serde_json::from_value(value)
        .context("legacy RID v1 profile could not be decoded")?;
    ensure!(legacy.schema_version == 1, "legacy RID schema mismatch");
    ensure!(legacy.level_curve_version == 1, "unsupported legacy RID level curve");
    ensure!(!legacy.identity.instance_id.trim().is_empty(), "legacy RID identity is empty");
    ensure!(legacy.identity.created_at_unix > 0, "legacy RID creation timestamp is invalid");
    ensure!(legacy.stats.len() == STAT_COUNT, "legacy RID profile must contain exactly 99 stat records");

    for (index, record) in legacy.stats.iter().enumerate() {
        let expected = index as u64 + 1;
        let expected_id = format!("asmaul_husna:{expected:02}");
        ensure!(
            record.get("id").and_then(Value::as_u64) == Some(expected)
                && record.get("catalogue_entry_id").and_then(Value::as_str) == Some(expected_id.as_str()),
            "legacy RID stat slot {expected} is invalid"
        );
    }

    let archive_path = legacy_archive_path(path);
    if archive_path.exists() {
        let existing = fs::read(&archive_path)
            .with_context(|| format!("could not read legacy archive {}", archive_path.display()))?;
        ensure!(
            existing == original_bytes,
            "existing legacy RID archive differs from current v1 profile; refusing overwrite"
        );
    } else {
        fs::write(&archive_path, original_bytes)
            .with_context(|| format!("could not preserve legacy RID profile at {}", archive_path.display()))?;
        File::open(&archive_path)?.sync_all()?;
    }

    let migrated_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_secs();
    let profile = RuhProfile::migrate_legacy(
        legacy.identity,
        legacy.xp_ledger,
        LegacyStatArchive {
            source_schema_version: 1,
            source_catalogue_id: "rocksoul.rid.asmaul_husna_inspired.v1".into(),
            migrated_at_unix,
            records: legacy.stats,
        },
    )
    .map_err(|error| anyhow!("legacy RID migration failed: {error}"))?;

    save(path, &profile).context("could not persist migrated RID v2 profile")?;
    Ok(profile)
}

/// Load an existing profile without overwriting it, or create the first profile.
pub fn initialize(path: &Path, identity: RuhIdentity) -> Result<(RuhProfile, bool)> {
    if path.exists() || backup_path(path).exists() {
        return Ok((load(path)?, false));
    }

    let profile = RuhProfile::new(identity)
        .map_err(|error| anyhow!("could not create RID profile: {error}"))?;
    save(path, &profile)?;
    Ok((profile, true))
}

/// Write a validated profile through a same-directory temporary file.
///
/// Existing data is moved to a backup before replacement. If replacement fails,
/// the previous profile is restored. This supports Windows, where rename does
/// not replace an existing destination file.
pub fn save(path: &Path, profile: &RuhProfile) -> Result<()> {
    profile
        .validate()
        .map_err(|error| anyhow!("refusing to persist invalid RID profile: {error}"))?;

    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("RID profile path has no parent directory"))?;
    fs::create_dir_all(parent).with_context(|| format!("could not create {}", parent.display()))?;

    let bytes = serde_json::to_vec_pretty(profile).context("could not serialize RID profile")?;
    let temporary = temporary_path(path)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .with_context(|| format!("could not create {}", temporary.display()))?;
    file.write_all(&bytes)
        .with_context(|| format!("could not write {}", temporary.display()))?;
    file.sync_all()
        .with_context(|| format!("could not sync {}", temporary.display()))?;
    drop(file);

    let backup = backup_path(path);
    let moved_previous = if path.exists() {
        if backup.exists() {
            fs::remove_file(&backup)
                .with_context(|| format!("could not remove stale backup {}", backup.display()))?;
        }
        fs::rename(path, &backup)
            .with_context(|| format!("could not stage previous RID profile {}", path.display()))?;
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
            .with_context(|| format!("could not replace RID profile {}", path.display()));
    }

    if moved_previous {
        let _ = fs::remove_file(backup);
    }

    if let Ok(directory) = File::open(parent) {
        let _ = directory.sync_all();
    }

    Ok(())
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.bak")
}

fn legacy_archive_path(path: &Path) -> PathBuf {
    path.with_extension("json.v1.bak")
}

fn temporary_path(path: &Path) -> Result<PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_nanos();
    let name = path
        .file_name()
        .ok_or_else(|| anyhow!("RID profile path has no file name"))?
        .to_string_lossy();
    let suffix = format!("{}-{nanos}.tmp", std::process::id());
    Ok(path.with_file_name(format!(".{name}.{suffix}")))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::rid::RuhIdentity;

    use super::{initialize, load, save};

    fn test_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "rocksoul-rid-test-{}-{nanos}/rid/profile.json",
            std::process::id()
        ))
    }

    fn identity(created_at_unix: u64) -> RuhIdentity {
        RuhIdentity {
            instance_id: "rocksoul-test-instance".into(),
            created_at_unix,
        }
    }

    #[test]
    fn initializes_once_and_preserves_identity() {
        let path = test_path();
        let (first, created) = initialize(&path, identity(1_800_000_000)).unwrap();
        assert!(created);

        let (second, created_again) = initialize(&path, identity(1_900_000_000)).unwrap();
        assert!(!created_again);
        assert_eq!(first, second);
        assert_eq!(second.identity().created_at_unix, 1_800_000_000);

        fs::remove_dir_all(path.ancestors().nth(2).unwrap()).unwrap();
    }

    #[test]
    fn saves_and_loads_validated_profile() {
        let path = test_path();
        let profile = crate::rid::RuhProfile::new(identity(1_800_000_000)).unwrap();

        save(&path, &profile).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(profile, loaded);

        fs::remove_dir_all(path.ancestors().nth(2).unwrap()).unwrap();
    }

    #[test]
    fn invalid_json_is_not_reinitialized_or_overwritten() {
        let path = test_path();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"{broken").unwrap();

        assert!(initialize(&path, identity(1_800_000_000)).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"{broken");

        fs::remove_dir_all(path.ancestors().nth(2).unwrap()).unwrap();
    }
}
