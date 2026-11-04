use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
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

        let data_dir = project.data_local_dir().to_path_buf();

        Ok(Self {
            config_dir: project.config_dir().to_path_buf(),
            config_file: project.config_dir().join("config.toml"),
            workspace_dir: data_dir.join("workspace"),
            memory_dir: data_dir.join("memory"),
            evolution_dir: data_dir.join("evolution"),
            logs_dir: data_dir.join("logs"),
            data_dir,
            cache_dir: project.cache_dir().to_path_buf(),
        })
    }
}
