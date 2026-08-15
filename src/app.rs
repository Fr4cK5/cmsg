use eyre::Result;

use crate::{
    cli::{Action, Cli},
    cmd_action::{CmdData, Count},
    config::Config,
    meta::MetadataRepo,
    walker::Walker,
};

/// The application(TM)
pub struct App {
    /// CLI arguments
    cli: Cli,

    /// Runtime configuration
    config: Config,

    /// The repo to manipulate the DB
    metadata_repo: MetadataRepo,
}

impl App {
    pub fn new(cli: Cli, config: Config, metadata_repo: MetadataRepo) -> Self {
        Self {
            cli,
            config,
            metadata_repo,
        }
    }

    /// Run the action
    pub fn run(&self) -> Result<()> {
        let walker = Walker::build_from(&self.cli)?;
        let mut parsed_files = walker.walk();
        parsed_files.sort();

        let cmd_data = CmdData {
            files: &parsed_files,
            output: self.cli.format,
            config: &self.config,
            repo: &self.metadata_repo,
        };

        match self.cli.action.as_ref().unwrap_or(&Action::default()) {
            Action::List(list) => list.run(&cmd_data),
            Action::Inspect => todo!(),
            Action::Commit(commit) => commit.run(&cmd_data),
            Action::Reset => todo!(),
            // Locate should print the base-dir/data-dir, the whole path to the back including the
            // commit hash
            Action::Locate => todo!(),
            Action::Count => Count::run(&cmd_data),
            Action::Clean(clean) => clean.run(&cmd_data),
        }?;

        Ok(())
    }
}
