use std::fmt::Write as _;

use eyre::{Result, eyre};

use crate::{cli::OutputFormat, cmd_action::CmdData, serialize_types::SerializableParsedFile};

/// The `ls` subcommand
#[derive(Debug, Clone, Default, clap::Parser)]
pub struct List {
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Copy the output to the system clipboard"
    )]
    pub copy: bool,
}

impl List {
    pub fn run(&self, data: &CmdData) -> Result<()> {
        match data.output {
            OutputFormat::Natural => {
                let mut buf = String::new();

                for file in &data.files.files {
                    _ = writeln!(&mut buf, "File {}", file.relative_path.display());

                    let mut max_len = file
                        .lines
                        .iter()
                        .map(|item| item.line)
                        .max()
                        .unwrap_or_default();

                    let mut max_width = 1usize;
                    while max_len > 10 {
                        max_width += 1;
                        max_len /= 10;
                    }

                    for line in file.lines.iter() {
                        _ = writeln!(
                            &mut buf,
                            "  Line {: >ln_width$}: {}",
                            line.line,
                            line.message,
                            ln_width = max_width
                        );
                    }

                    buf.push('\n');
                }

                println!("{buf}");
            }
            OutputFormat::Vim => {
                let mut buf = String::new();

                for file in &data.files.files {
                    for line in &file.lines {
                        _ = writeln!(
                            &mut buf,
                            "{}:{}={}",
                            file.relative_path.display(),
                            line.line,
                            line.message
                        );
                    }
                }

                println!("{buf}");
            }
            OutputFormat::Json => {
                let files = data
                    .files
                    .files
                    .iter()
                    .map(|file| SerializableParsedFile::from(file.clone()))
                    .collect::<Vec<_>>();

                let json = serde_json::to_string_pretty(&files)
                    .map_err(|err| eyre!("{err}: Failed to serialize parsed files to json"))?;

                println!("{json}");
            }
        }

        Ok(())
    }
}
