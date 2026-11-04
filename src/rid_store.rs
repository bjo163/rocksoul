use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow};

use crate::rid::{RuhIdentity, RuhProfile};

/// Load and validate an existing local RID profile.
pub fn load(path: &Path) -> Result<RuhProfile> {
    if !path.exists() {
        let backup = backup_path(path);
        if backup.is_file() {
            fs::rename(&backup, path).with_context(|| {
                format!("could not recover RID backup {}", backup.display())
            })?;
        } else {
            return Err(anyhow!("RID profile does not exist at {}", path.display()));
        }
    }

    let bytes = fs::read(path).with_context(|| format!("could not read {}", path.display()))?;
    let profile: RuhProfile = serde_json::from_slice(&bytes)
        .with_context(|| format!("RID profile is not valid JSON: {}", path.display()))?;
    profile
        .validate()
        .map_err(|error| anyhow!("RID profile validation failed: {error}"))?;
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
    fs::create_dir_all(parent)
        .with_context(|| format!("could not create {}", parent.display()))?;

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
        return Err(error).with_context(|| format!("could not replace RID profile {}", path.display()));
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
