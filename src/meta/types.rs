use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    /// The backup's random SHA-256 digest as a 64-char hex string
    pub digest: String,

    /// The base directory such as ~/.local/share/cmsg
    pub data_directory: PathBuf,

    /// The backup root such as ~/.local/share/cmsg/<64-char-hex-string>
    pub backup_root: PathBuf,
}
