use std::{
    env::{self},
    fmt::Display,
    fs,
    path::PathBuf,
};

use eyre::{OptionExt, Result, eyre};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum StorageStrategy {
    #[default]
    GlobalFallback,
    Global,
    DotGitOnly,
}

impl Display for StorageStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GlobalFallback => f.write_str("GlobalFallback"),
            Self::Global => f.write_str("DotGitOnly"),
            Self::DotGitOnly => f.write_str("DotGitOnly"),
        }
    }
}

impl StorageStrategy {
    pub fn locate_data_dir(self) -> Option<PathBuf> {
        match self {
            StorageStrategy::GlobalFallback => Self::dotgit_data().or_else(Self::user_home_data),
            StorageStrategy::Global => Self::user_home_data(),
            StorageStrategy::DotGitOnly => Self::dotgit_data(),
        }
    }

    pub fn dotgit_data() -> Option<PathBuf> {
        let cwd = env::current_dir().ok()?;
        let parts = cwd.iter().collect::<Vec<_>>();
        let mut parts = (1..=parts.len())
            .flat_map(|i| {
                let path = parts.iter().take(i).collect::<PathBuf>().join(".git");
                fs::exists(&path)
                    .ok()
                    .and_then(|exists| exists.then_some(path))
                    .map(|path| path.join("cmsg"))
            })
            .collect::<Vec<_>>();

        parts.sort_by_key(|item| item.iter().count());
        parts.last().cloned()
    }

    pub fn user_home_data() -> Option<PathBuf> {
        env::home_dir().map(|dir| {
            if cfg!(target_os = "windows") {
                dir.join("AppData/Local/cmsg")
            } else {
                dir.join(".local/share/cmsg")
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage_strategy: StorageStrategy,

    #[serde(skip)]
    pub data_directory: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_directory: StorageStrategy::default()
                .locate_data_dir()
                .expect("Unable to find any suitable storage directory.\nThis error signifies that there was no project-local .git directory and your home directory could not be found."),
            storage_strategy: StorageStrategy::default(),
        }
    }
}

impl Config {
    pub fn load(storage_strategy: StorageStrategy) -> Result<Self> {
        let path = storage_strategy
            .locate_data_dir()
            .ok_or_eyre(eyre!("Unable to locate suitable data directory according to storage strategy {storage_strategy}"))?;

        Ok(Self {
            storage_strategy,
            data_directory: path,
        })
    }
}
