use std::{
    env::{self},
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use eyre::{ContextCompat, OptionExt, Result, eyre};
use serde::{Deserialize, Serialize};

use crate::pathutil;

/// The storage strategy defines how to pick a data directory, and implements the necessary things
/// to locate a suitable one for any valid state.
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
    pub fn locate_data_dir(self, path: &Path) -> Option<PathBuf> {
        match self {
            StorageStrategy::GlobalFallback => Self::dotgit_data(path)
                .or_else(|_| Self::user_home_data())
                .ok(),
            StorageStrategy::Global => Self::user_home_data().ok(),
            StorageStrategy::DotGitOnly => Self::dotgit_data(path).ok(),
        }
    }

    pub fn dotgit_data(path: &Path) -> Result<PathBuf> {
        let parts = path.iter().collect::<Vec<_>>();
        // This needs to be 2.. because if we start from 1..,
        // we'd get paths like this: C:.git, which, for some reason, windows says exist...
        // This isn't a problem on linux, where /.git would be a directory rather than a volume
        // label which would probably not exist.
        let base_offset = if cfg!(target_os = "windows") { 2 } else { 1 };
        let mut parts = (base_offset..=parts.len())
            .flat_map(|i| {
                let path = parts.iter().take(i).collect::<PathBuf>().join(".git");
                fs::exists(&path)
                    .ok()
                    .and_then(|exists| exists.then_some(path))
                    .map(|path| path.join("cmsg"))
            })
            .collect::<Vec<_>>();

        parts.sort_by_key(|item| item.iter().count());

        parts
            .last()
            .wrap_err_with(|| eyre!("Unable to find suitable project-local data directory"))
            .and_then(pathutil::normalize)
    }

    pub fn user_home_data() -> Result<PathBuf> {
        env::home_dir()
            .ok_or_eyre(eyre!("Unable to get home directory of current user"))
            .and_then(|dir| {
                if cfg!(target_os = "windows") {
                    pathutil::normalize(dir.join("AppData/Local/cmsg/data"))
                } else {
                    pathutil::normalize(dir.join(".local/share/cmsg/data"))
                }
            })
    }

    pub fn user_home_db() -> eyre::Result<PathBuf> {
        env::home_dir()
            .ok_or_eyre(eyre!("Unable to get home directory of current user"))
            .and_then(|dir| {
                if cfg!(target_os = "windows") {
                    pathutil::normalize(dir.join("AppData/Local/cmsg/data.db"))
                } else {
                    pathutil::normalize(dir.join(".local/share/cmsg/data.db"))
                }
            })
    }
}

/// The runtime config
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
                .locate_data_dir(&env::current_dir().expect("Unable to get cwd"))
                .expect("Unable to find any suitable storage directory.\nThis error signifies that there was no project-local .git directory and your home directory could not be found."),
            storage_strategy: StorageStrategy::default(),
        }
    }
}

impl Config {
    pub fn load(path: &Path, storage_strategy: StorageStrategy) -> Result<Self> {
        let path = storage_strategy
            .locate_data_dir(path)
            .ok_or_eyre(eyre!("Unable to locate suitable data directory according to storage strategy {storage_strategy}"))?;

        Ok(Self {
            storage_strategy,
            data_directory: path,
        })
    }
}
