use std::fs;

use eyre::{Result, eyre};

use crate::{
    cli::OutputFormat,
    cmd_action::CmdData,
    meta::MetadataRepo,
    trie::{PrefixTrie, TrieLookupResult},
};

/// The `drop` subcommand
#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Drop {
    /// The hash of the commit to be dropped
    pub input_hashes: Vec<String>,
}

impl Drop {
    pub fn run(&self, data: &CmdData) -> Result<()> {
        let hashes = data
            .repo
            .transaction(|tx| MetadataRepo::fetch_backup_hashes(tx, &data.config.data_directory))?;

        let trie = PrefixTrie::from(hashes);

        let hashes = self
            .input_hashes
            .iter()
            .map(|hash| match trie.get_by_prefix(hash.to_uppercase()) {
                TrieLookupResult::None => {
                    Err(eyre!("Input did not match any known hashes: {hash}"))
                }
                TrieLookupResult::Ambiguous => Err(eyre!("Input is ambiguous: {hash}",)),
                TrieLookupResult::Unique(value) => Ok(value),
            })
            .collect::<Vec<_>>();

        let errors = hashes
            .iter()
            .flat_map(|item| {
                if let Err(e) = item {
                    return Some(e.to_string());
                }
                None
            })
            .collect::<Vec<_>>();

        if !errors.is_empty() {
            return Err(eyre!("\n{}", errors.join("\n")));
        }

        let hashes = hashes.into_iter().flatten().collect::<Vec<_>>();

        data.repo.transaction(|tx| {
            for hash in &hashes {
                MetadataRepo::remove_commit(tx, hash)?;
            }

            MetadataRepo::gc_dangling_data_directories(tx)?;
            for hash in &hashes {
                fs::remove_dir_all(data.config.data_directory.join(hash))?;
            }
            Ok(())
        })?;

        match data.output {
            OutputFormat::Natural => {
                println!("Dropped following hashes:");
                for hash in hashes {
                    println!("  {hash}");
                }
            }
            OutputFormat::Vim => {
                for hash in hashes {
                    println!("{hash}");
                }
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&hashes)?);
            }
        }

        Ok(())
    }
}
