use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root_dir: PathBuf,
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub env_file: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub memory_dir: PathBuf,
    pub evolution_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub rid_file: PathBuf,
}

impl AppPaths {
    /// Resolve every mutable path relative to the executable directory.
    ///
    /// This is intentionally portable: no Rocksoul configuration or runtime
    /// state is written to AppData, Documents, or another user-profile folder.
    pub fn discover() -> Result<Self> {
        let executable_path =
            std::env::current_exe().context("could not determine the current executable path")?;
        let root_dir = executable_path
            .parent()
            .context("the current executable has no parent directory")?
            .to_path_buf();

        Ok(Self::from_root(root_dir))
    }

    pub(crate) fn from_root(root_dir: PathBuf) -> Self {
        let config_dir = root_dir.join("config");
        let data_dir = root_dir.join("data");

        Self {
            root_dir: root_dir.clone(),
            config_file: config_dir.join("config.toml"),
            env_file: root_dir.join(".env"),
            workspace_dir: root_dir.join("workspace"),
            memory_dir: data_dir.join("memory"),
            evolution_dir: root_dir.join("evolution"),
            logs_dir: root_dir.join("logs"),
            rid_file: data_dir.join("identity").join("rid.json"),
            cache_dir: root_dir.join("cache"),
            data_dir,
            config_dir,
        }
    }
}

/// Load only the dotenv file in the Rocksoul installation root.
/// Existing process environment variables intentionally take precedence.
pub fn load_executable_dotenv() -> Result<bool> {
    let executable_path =
        std::env::current_exe().context("could not determine the current executable path")?;
    let env_file = dotenv_path_for_executable(&executable_path)?;

    if !env_file.is_file() {
        return Ok(false);
    }

    dotenvy::from_path(&env_file)
        .with_context(|| format!("could not read {}", env_file.display()))?;

    Ok(true)
}

fn dotenv_path_for_executable(executable_path: &Path) -> Result<PathBuf> {
    let directory = executable_path
        .parent()
        .context("the current executable has no parent directory")?;
    Ok(directory.join(".env"))
}

#[cfg(test)]
mod tests {
    use super::{AppPaths, dotenv_path_for_executable};
    use std::path::Path;

    #[test]
    fn dotenv_is_resolved_from_installation_root() {
        let path = dotenv_path_for_executable(Path::new("C:/Rocksoul/rocksoul.exe"))
            .expect("executable path should have a parent");
        assert_eq!(path, Path::new("C:/Rocksoul/.env"));
    }

    #[test]
    fn all_mutable_paths_are_inside_the_installation_root() {
        let paths = AppPaths::from_root(Path::new("C:/Rocksoul").to_path_buf());

        assert_eq!(paths.env_file, Path::new("C:/Rocksoul/.env"));
        assert_eq!(
            paths.config_file,
            Path::new("C:/Rocksoul/config/config.toml")
        );
        assert_eq!(
            paths.rid_file,
            Path::new("C:/Rocksoul/data/identity/rid.json")
        );

        for path in [
            &paths.config_dir,
            &paths.config_file,
            &paths.env_file,
            &paths.data_dir,
            &paths.cache_dir,
            &paths.workspace_dir,
            &paths.memory_dir,
            &paths.evolution_dir,
            &paths.logs_dir,
            &paths.rid_file,
        ] {
            assert!(path.starts_with(&paths.root_dir), "{path:?} escaped root");
        }
    }
}
