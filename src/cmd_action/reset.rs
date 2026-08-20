use std::fs;

use eyre::{Result, eyre};

use crate::{cli::OutputFormat, cmd_action::CmdData, fs_ext, meta::MetadataRepo, trie::PrefixTrie};

/// The `reset` subcommand
#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Reset {
    /// The hash to reset to
    pub input_hash: String,

    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Remove the commit after resetting, will only be deleted if the reset is successful"
    )]
    pub remove_commit: bool,
}

impl Reset {
    pub fn run(&self, data: &CmdData) -> Result<()> {
        let hashes = data
            .repo
            .transaction(|tx| MetadataRepo::fetch_backup_hashes(tx, &data.config.data_directory))?;

        let trie = PrefixTrie::from(hashes);

        let found_hashes = match trie.get_by_prefix_all(self.input_hash.to_uppercase()) {
            Some(items) if items.len() == 1 => items,
            Some(items) => {
                eprintln!("Input is ambiguous; it matched the following hashes: ");
                for hash in items {
                    eprintln!("{hash}");
                }
                return Err(eyre!("Input is ambiguous"));
            }
            None => return Err(eyre!("Input did not match any known hashes")),
        };

        let Some(hash) = found_hashes.first() else {
            return Err(eyre!("Unable to get hash, even though it exists."));
        };

        let src = data.config.data_directory.join(hash);
        let destination = &data.config.working_directory;

        fs_ext::copy_all_children(src, destination)?;

        match data.output {
            OutputFormat::Natural => println!("Successfully reset to {hash}"),
            OutputFormat::Vim => println!("{hash}"),
            OutputFormat::Json => println!(r#""{hash}""#),
        }

        if self.remove_commit {
            data.repo.transaction(|tx| {
                MetadataRepo::remove_commit(tx, hash)?;
                MetadataRepo::gc_dangling_data_directories(tx)?;
                fs::remove_dir_all(data.config.data_directory.join(hash))?;
                Ok(())
            })?;
        }

        Ok(())
    }
}
