use std::{error::Error, path::PathBuf};

use serde::{Deserialize, Serialize};

/// A single backup, created when invoking the `commit` command / from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitData {
    /// The backup's random SHA-256 hash as a 64-char hex string
    pub hash: String,

    /// The base directory such as ~/.local/share/cmsg
    pub data_directory: PathBuf,

    /// The backup root such as ~/.local/share/cmsg/<64-char-hex-string>
    pub backup_root: PathBuf,
}

/// A single file
#[derive(Debug)]
pub struct RollbackData {
    /// The original source directory
    pub source: PathBuf,

    /// The `.cmsg`-appended safe-write backup
    pub destination: PathBuf,

    /// The original error that caused this failure, if any.
    pub error: Option<Box<dyn Error>>,
}
