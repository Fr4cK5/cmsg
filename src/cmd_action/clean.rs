use std::{fs, path::PathBuf};

use eyre::{Result, eyre};
use rusqlite::named_params;

use crate::{cli::OutputFormat, cmd_action::CmdData, meta::MetadataRepo};

/// The `clean` subcommand
#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Clean {
    #[arg(
        short = 'a',
        long,
        default_value_t = false,
        help = "Remove *all* data directories instead of just the one's belonging to the local project."
    )]
    pub remove_all: bool,
}

impl Clean {
    pub fn run(&self, data: &CmdData) -> Result<()> {
        if self.remove_all {
            self.remove_all(data)
        } else {
            self.remove_local(data)
        }
    }

    fn fetch_metadata(&self, data: &CmdData) -> Result<Option<(Vec<String>, i64)>> {
        data.repo.transaction(|tx| {
            MetadataRepo::fetch_hashes_for_data_directory(tx, &data.config.data_directory)
        })
    }

    fn remove_local(&self, data: &CmdData) -> Result<()> {
        let (commit_hashes, data_directory_id) = match self.fetch_metadata(data) {
            Ok(None) => return Ok(()),
            Ok(Some((rows, id))) => (rows, id),
            Err(e) => return Err(e),
        };

        data.repo.transaction(|tx| {
            let mut prepared = tx.prepare("delete from backup_entry where hash = :hash")?;
            for commit_hash in &commit_hashes {
                let dir = PathBuf::from(&data.config.data_directory).join(commit_hash);
                if fs::exists(&dir)?
                    && let Err(err) = fs::remove_dir_all(&dir)
                {
                    return Err(eyre!("{err}: Failed to remove entry '{}'", dir.display()));
                }

                prepared.execute(named_params! {
                    ":hash": &commit_hash,
                })?;
            }

            let mut prepared = tx.prepare("delete from data_directory where id = :id")?;
            prepared.execute(named_params! {
                ":id": data_directory_id,
            })?;

            Ok(())
        })?;

        match data.output {
            OutputFormat::Natural => {
                println!("Removed the following entries:");
                for commit_hash in commit_hashes {
                    println!("  {commit_hash}");
                }
            }
            OutputFormat::Vim => {
                for commit_hash in commit_hashes {
                    println!("{commit_hash}");
                }
            }
            OutputFormat::Json => {
                if let Ok(json) = serde_json::to_string_pretty(&commit_hashes) {
                    println!("{json}");
                }
            }
        }

        Ok(())
    }

    fn remove_all(&self, data: &CmdData) -> Result<()> {
        let directories = data.repo.transaction(|tx| {
            let mut prepared = tx.prepare("select path from data_directory")?;
            let rows = prepared.query_map([], |row| row.get::<_, String>(0))?;
            let mut directories = Vec::new();
            for row in rows.flatten() {
                directories.push(row)
            }

            Ok(directories)
        })?;

        data.repo.transaction(|tx| {
            let mut prepared = tx.prepare("delete from data_directory where path = :path")?;
            for dir in &directories {
                if fs::exists(dir).unwrap_or_default()
                    && let Err(e) = fs::remove_dir_all(dir)
                {
                    eprintln!("{e}: Failed to remove data directory '{dir}'");
                } else {
                    prepared.execute(named_params! {
                        ":path": &dir,
                    })?;
                }
            }

            Ok(())
        })?;

        match data.output {
            OutputFormat::Natural => {
                println!("Removed the following data directories:");
                for dir in directories {
                    println!("  {dir}");
                }
            }
            OutputFormat::Vim => {
                for dir in directories {
                    println!("{dir}");
                }
            }
            OutputFormat::Json => {
                if let Ok(json) = serde_json::to_string_pretty(&directories) {
                    println!("{json}");
                }
            }
        }

        Ok(())
    }
}
