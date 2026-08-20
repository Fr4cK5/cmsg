use std::fs;

use eyre::{Result, eyre};

use crate::{
    cmd_action::CmdData,
    meta::MetadataRepo,
    trie::{PrefixTrie, TrieLookupResult},
};

/// The `drop` subcommand
#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Drop {
    /// The hash of the commit to be dropped
    pub input_hash: String,

    // TODO: Make the input hash a Vec<String> and make it accept multiple hashes?
}

impl Drop {
    pub fn run(&self, data: &CmdData) -> Result<()> {
        let hashes = data
            .repo
            .transaction(|tx| MetadataRepo::fetch_backup_hashes(tx, &data.config.data_directory))?;

        let trie = PrefixTrie::from(hashes);
        let value = match trie.get_by_prefix(self.input_hash.clone()) {
            TrieLookupResult::None => return Err(eyre!("Input did not match any known hashes")),
            TrieLookupResult::Ambiguous => return Err(eyre!("Input is ambiguous")),
            TrieLookupResult::Unique(value) => value,
        };

        data.repo.transaction(|tx| {
            MetadataRepo::remove_commit(tx, value)?;
            MetadataRepo::gc_dangling_data_directories(tx)?;
            fs::remove_dir_all(data.config.data_directory.join(value))?;
            Ok(())
        })?;

        Ok(())
    }
}
