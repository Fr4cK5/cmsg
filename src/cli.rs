use clap::{CommandFactory, Parser};
use eyre::Result;
use std::{fmt::Display, path::PathBuf};

use crate::{
    cmd_action::{Clean, Commit, List},
    parser::ParsedFiles,
    writer::{json::JsonFormatter, natural::NaturalFormatter, vim::VimFormatter},
};

#[derive(Debug, Clone, clap::Parser)]
#[command(
    version = "0.0.1",
    about = "cmsg: track your work in plain text directly where you write it"
)]
pub struct Cli {
    #[arg(
        short = 'I',
        long,
        default_value_t = false,
        help = "Disable respecting files/glob patterns listen in ignore files eg .gitignore, .ignore, ..."
    )]
    pub no_ignore: bool,

    #[arg(
        short = 'H',
        long,
        default_value_t = false,
        help = "Include hidden files in the search"
    )]
    pub include_hidden: bool,

    #[arg(short = 'd', long, default_value_t = String::from("."), help = "Path from which to start recursively searching files")]
    pub base_directory: String,

    #[arg(short = 'f', long, default_value_t = Default::default(), help = "Set the output format")]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub action: Option<Action>,
}

impl Cli {
    pub fn parse_and_validate() -> Self {
        Self::try_parse()
            .and_then(Self::validate)
            .unwrap_or_else(|e| e.exit())
    }

    fn validate(self) -> Result<Self, clap::Error> {
        let target_dir = PathBuf::from(&self.base_directory);

        if !target_dir.is_dir() {
            return Err(Self::command().error(
                clap::error::ErrorKind::InvalidValue,
                format!(
                    "provided path must point to a directory: '{}'. If it's a directory, you might not have sufficient permissions to access it",
                    target_dir.display()
                )
                .trim(),
            ));
        }

        Ok(self)
    }
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Natural,
    Vim,
    Json,
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Self::Natural => "natural",
            Self::Vim => "vim",
            Self::Json => "json",
        })
    }
}

impl OutputFormat {
    pub fn format(&self, entries: &ParsedFiles) -> Result<String> {
        Ok(match *self {
            Self::Natural => NaturalFormatter::format(entries),
            Self::Vim => VimFormatter::format(entries),
            Self::Json => JsonFormatter::format(entries)?,
        })
    }
}

#[derive(Debug, Clone, clap::Parser)]
pub enum Action {
    #[command(name = "ls", about = "List all occurences of .cmsg markers")]
    List(List),

    #[command(
        name = "commit",
        about = "List all occurences of .cmsg markers, remove them from the code, and return a hash for undoing the commit should this mess up the code"
    )]
    Commit(Commit),

    #[command(name = "undo", about = "Undo a previous commit based on a hash")]
    Undo,

    #[command(name = "count", about = "Count all occurences of .cmsg markers")]
    Count,

    #[command(
        name = "clean",
        about = "Clean all data directories, reclaiming some storage space"
    )]
    Clean(Clean),
}

impl Default for Action {
    fn default() -> Self {
        Self::List(List::default())
    }
}
