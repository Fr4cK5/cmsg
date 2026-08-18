use eyre::Result;

use crate::{cli::OutputFormat, cmd_action::CmdData, meta::MetadataRepo};

/// The `inspect` subcommand
#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Inspect;

impl Inspect {
    pub fn run(data: &CmdData) -> Result<()> {
        let dirs = data
            .repo
            .transaction(MetadataRepo::fetch_data_dirs_and_backups)?;

        match data.output {
            OutputFormat::Natural => {
                for (data_dir, backups) in &dirs {
                    println!("Data directory: {data_dir}");
                    for backup in backups {
                        println!("  {backup}");
                    }
                    println!();
                }
            }
            OutputFormat::Vim => {
                for (data_dir, backups) in &dirs {
                    for backup in backups {
                        println!("{backup} {data_dir}/{backup}");
                    }
                }
            }
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&dirs)?;
                println!("{json}");
            }
        }

        Ok(())
    }
}
