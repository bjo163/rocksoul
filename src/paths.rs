use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub executable_dir: PathBuf,
    pub env_file: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub memory_dir: PathBuf,
    pub evolution_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self> {
        let project = ProjectDirs::from("com", "Rocksoul", "Rocksoul")
            .context("could not resolve the operating system's application directories")?;

        let executable_path =
            std::env::current_exe().context("could not determine the current executable path")?;
        let executable_dir = executable_path
            .parent()
            .context("the current executable has no parent directory")?
            .to_path_buf();
        let env_file = executable_dir.join(".env");
        let data_dir = project.data_local_dir().to_path_buf();

        Ok(Self {
            config_dir: project.config_dir().to_path_buf(),
            config_file: project.config_dir().join("config.toml"),
            executable_dir,
            env_file,
            workspace_dir: data_dir.join("workspace"),
            memory_dir: data_dir.join("memory"),
            evolution_dir: data_dir.join("evolution"),
            logs_dir: data_dir.join("logs"),
            data_dir,
            cache_dir: project.cache_dir().to_path_buf(),
        })
    }
}

/// Load only the dotenv file colocated with the executable, never a file
/// discovered from the caller's current working directory.
pub fn load_executable_dotenv() -> Result<bool> {
    let executable_path =
        std::env::current_exe().context("could not determine the current executable path")?;
    let env_file = dotenv_path_for_executable(&executable_path)?;

    if !env_file.is_file() {
        return Ok(false);
    }

    // dotenvy::from_path preserves variables already present in the process environment.
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
    use super::dotenv_path_for_executable;
    use std::path::Path;

    #[test]
    fn dotenv_is_resolved_next_to_the_executable() {
        let path = dotenv_path_for_executable(Path::new("/opt/rocksoul/rocksoul.exe"))
            .expect("executable path should have a parent");
        assert_eq!(path, Path::new("/opt/rocksoul/.env"));
    }
}
