use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub const PREFERENCES_SCHEMA_VERSION: u16 = 1;
pub const PREFERENCES_FILE: &str = "tui-preferences.v1.json";

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DensityMode {
    Compact,
    Comfortable,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocaleMode {
    English,
    Indonesian,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct TuiPreferences {
    pub schema_version: u16,
    pub theme: ThemeMode,
    pub density: DensityMode,
    pub locale: LocaleMode,
}

impl Default for TuiPreferences {
    fn default() -> Self {
        Self {
            schema_version: PREFERENCES_SCHEMA_VERSION,
            theme: ThemeMode::Dark,
            density: DensityMode::Compact,
            locale: LocaleMode::English,
        }
    }
}

impl TuiPreferences {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.is_file() {
            return Ok(Self::default());
        }
        let bytes = fs::read(path).with_context(|| "could not read TUI preferences")?;
        let preferences: Self =
            serde_json::from_slice(&bytes).with_context(|| "invalid TUI preferences")?;
        ensure!(
            preferences.schema_version == PREFERENCES_SCHEMA_VERSION,
            "unsupported TUI preferences schema"
        );
        Ok(preferences)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        ensure!(
            self.schema_version == PREFERENCES_SCHEMA_VERSION,
            "unsupported TUI preferences schema"
        );
        if path.exists() {
            let existing = Self::load(path)?;
            ensure!(
                existing.schema_version == PREFERENCES_SCHEMA_VERSION,
                "refusing to overwrite an unknown TUI preferences schema"
            );
        }
        let parent = path.parent().context("preferences path has no parent")?;
        fs::create_dir_all(parent).with_context(|| "could not create preferences directory")?;
        let temporary = path.with_extension("json.tmp");
        let backup = path.with_extension("json.bak");
        if temporary.exists() {
            fs::remove_file(&temporary)
                .with_context(|| "could not remove stale preferences temp file")?;
        }
        let bytes = serde_json::to_vec_pretty(self)?;
        let mut file =
            File::create(&temporary).with_context(|| "could not create preferences temp file")?;
        file.write_all(&bytes)
            .with_context(|| "could not write preferences temp file")?;
        file.sync_all()
            .with_context(|| "could not sync preferences temp file")?;
        drop(file);

        if path.exists() {
            if backup.exists() {
                fs::remove_file(&backup)
                    .with_context(|| "could not remove stale preferences backup")?;
            }
            fs::rename(path, &backup).with_context(|| "could not preserve previous preferences")?;
            if let Err(error) = fs::rename(&temporary, path) {
                let _ = fs::rename(&backup, path);
                return Err(error).with_context(|| "could not replace TUI preferences");
            }
            fs::remove_file(&backup).with_context(|| "could not remove preferences backup")?;
        } else {
            fs::rename(&temporary, path).with_context(|| "could not publish TUI preferences")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DensityMode, LocaleMode, PREFERENCES_FILE, PREFERENCES_SCHEMA_VERSION, ThemeMode,
        TuiPreferences,
    };
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "rocksoul-tui-preferences-{}-{}.json",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn missing_preferences_use_safe_portable_defaults() {
        let path = temp_path();
        let preferences = TuiPreferences::load(&path).unwrap();
        assert_eq!(preferences, TuiPreferences::default());
        assert_eq!(PREFERENCES_FILE, "tui-preferences.v1.json");
        assert_eq!(preferences.schema_version, PREFERENCES_SCHEMA_VERSION);
    }

    #[test]
    fn preferences_round_trip_all_presentation_dimensions() {
        let path = temp_path();
        let preferences = TuiPreferences {
            schema_version: PREFERENCES_SCHEMA_VERSION,
            theme: ThemeMode::Light,
            density: DensityMode::Comfortable,
            locale: LocaleMode::Indonesian,
        };
        preferences.save(&path).unwrap();
        assert_eq!(TuiPreferences::load(&path).unwrap(), preferences);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn unknown_future_schema_is_never_overwritten() {
        let path = temp_path();
        let future = TuiPreferences {
            schema_version: PREFERENCES_SCHEMA_VERSION + 1,
            ..TuiPreferences::default()
        };
        let bytes = serde_json::to_vec(&future).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert!(TuiPreferences::load(&path).is_err());
        assert!(TuiPreferences::default().save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), bytes);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn malformed_preferences_are_not_silently_replaced() {
        let path = temp_path();
        fs::write(&path, b"{not-json").unwrap();
        assert!(TuiPreferences::load(&path).is_err());
        assert!(TuiPreferences::default().save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"{not-json");
        fs::remove_file(path).unwrap();
    }
}
