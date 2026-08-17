use std::path::PathBuf;

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
