use clap::{CommandFactory, Parser};
use eyre::Result;
use std::{fmt::Display, path::PathBuf};

use crate::cmd_action::{clean::Clean, commit::Commit, list::List, locate::Locate, reset::Reset};

/// The basic flag definitions all the globally available flags.
#[derive(Debug, Clone, clap::Parser)]
#[command(
    version = "0.0.1",
    about = "cmsg: track your work in plain text right then and there"
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

/// The output format
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

// TODO: Drop subcommand to drop a specific commit hash

/// An action, invocable from the CLI
#[derive(Debug, Clone, clap::Parser)]
pub enum Action {
    /// The `ls` command
    ///
    /// Output format:
    /// File <relative-path>
    ///   Line <line>: <msg>
    ///   ...
    ///
    /// ...
    #[command(
        name = "ls",
        alias = "l",
        about = "List all occurences of .cmsg markers"
    )]
    List(List),

    /// The `inspect` command
    ///
    /// Output format:
    /// <hash> <data-dir>/<hash>
    /// ...
    #[command(name = "inspect", alias = "i", about = "Inspect the metadata database")]
    Inspect,

    /// The `commit` command
    ///
    /// Output format:
    /// Reset to <hash>
    #[command(
        name = "commit",
        alias = "c",
        about = "List all occurences of .cmsg markers, remove them from the code, and return a hash to reset to should this mess up the code"
    )]
    Commit(Commit),

    /// The `reset` command
    ///
    /// Output format:
    /// Reset to <hash>
    #[command(
        name = "reset",
        about = "Reset to some previously commited state based on a hash"
    )]
    Reset(Reset),

    /// The `count` command
    ///
    /// Output format:
    /// <count>
    #[command(name = "count", about = "Count all occurences of .cmsg markers")]
    Count,

    /// The `locate` command
    ///
    /// Output format w/ specified hash:
    /// <data-dir>/<hash>
    ///
    /// Output format w/o specified hash:
    /// <data-dir>
    #[command(
        name = "locate",
        about = "Show the current project's data directory. I the hash is specified, it is used as a prefix filter for all known hashes"
    )]
    Locate(Locate),

    /// The `clean` command
    ///
    /// Output format:
    /// Removed the following data directories:
    ///   <data-directory>
    ///   ...
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
