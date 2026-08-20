use eyre::{Result, eyre};

use crate::{
    cli::OutputFormat,
    cmd_action::CmdData,
    meta::MetadataRepo,
    trie::{PrefixTrie, TrieLookupResult},
};

/// The `locate` subcommand
#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Locate {
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Allow ambiguous patterns to match, this enables the possibility of multiple hashes being printed to stdout. If used without an input hash, this will print all *local* hashes"
    )]
    pub allow_ambiguous: bool,
    pub hash: Option<String>,
}

impl Locate {
    pub fn run(&self, data: &CmdData) -> Result<()> {
        let paths = match &self.hash {
            Some(input_hash) => {
                let hashes = data.repo.transaction(|tx| {
                    MetadataRepo::fetch_backup_hashes(tx, &data.config.data_directory)
                })?;

                let normalized_hashes = hashes
                    .into_iter()
                    .map(|item| item.to_uppercase())
                    .collect::<Vec<_>>();

                let trie = PrefixTrie::from(normalized_hashes);

                if self.allow_ambiguous {
                    trie.get_by_prefix_all(input_hash.to_uppercase())
                        .map(|hashes| {
                            hashes
                                .into_iter()
                                .map(|hash| data.config.data_directory.join(hash))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                } else {
                    let found_hash = match trie.get_by_prefix(input_hash.to_uppercase()) {
                        TrieLookupResult::None => {
                            return Err(eyre!("Input did not match any known hashes"));
                        }
                        TrieLookupResult::Ambiguous => {
                            return Err(eyre!(
                                "Input is ambiguous; use --allow-ambiguous(-a) to get all matching hashes"
                            ));
                        }
                        TrieLookupResult::Unique(value) => value,
                    };

                    vec![data.config.data_directory.join(found_hash)]
                }
            }
            None if self.allow_ambiguous => data
                .repo
                .transaction(|tx| {
                    MetadataRepo::fetch_backup_hashes(tx, &data.config.data_directory)
                })?
                .into_iter()
                .map(|hash| data.config.data_directory.join(hash))
                .collect::<Vec<_>>(),
            None => vec![data.config.data_directory.clone()],
        };

        match data.output {
            OutputFormat::Natural => {
                if paths.len() == 1
                    && let Some(path) = paths.first()
                {
                    println!(
                        "The current project's data directory is: {}",
                        path.display()
                    );
                } else {
                    if paths.is_empty() {
                        println!("Input did not match any known hashes");
                    } else {
                        println!("The following data directories were found:");
                        for path in paths {
                            println!("  {}", path.display());
                        }
                    }
                }
                println!();
            }
            OutputFormat::Vim => {
                for path in paths {
                    println!("{}", path.display());
                }
            }
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&paths)?;
                println!("{json}");
            }
        }

        Ok(())
    }
}
