use std::path::PathBuf;

use eyre::{Result, eyre};

// TODO: the idea here is as follows: we'll have a global config file somethere in the user's $HOME,
// but of course also allow for local config files. Probably gonna whip out my beloved serde_json.
// Perhaps we should also walk up the current directory tree, to find the closest config file.
// And i suppose I could name the file just `cmsg.json`
#[derive(Debug, Clone)]
pub struct Config {
    pub data_directory: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // TODO: This will eventually change to something like $HOME/{.cache,AppData/Local}/cmsg
            data_directory: PathBuf::from(".git/cmsg"),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        Err(eyre!("config loading is not yet implemented"))
    }
}
