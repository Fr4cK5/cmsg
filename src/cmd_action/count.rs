use eyre::Result;

use crate::{cli::OutputFormat, cmd_action::CmdData};

/// The `count` subcommand
#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Count;

impl Count {
    pub fn run(data: &CmdData) -> Result<()> {
        let file_count = data.files.files.len();
        let line_count = data
            .files
            .files
            .iter()
            .map(|item| item.lines.len())
            .sum::<usize>();

        match data.output {
            OutputFormat::Natural => {
                println!("Files   : {}\nMessages: {}", file_count, line_count);
            }
            OutputFormat::Vim => {
                println!("files={}\nlines={}", file_count, line_count);
            }
            OutputFormat::Json => {
                println!(r#"{{ "files": {}, "lines": {} }}"#, file_count, line_count)
            }
        }

        Ok(())
    }
}
