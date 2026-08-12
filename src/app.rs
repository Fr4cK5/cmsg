use eyre::Result;

use crate::{
    cli::{Action, Cli},
    cmd_action::{CmdData, Count},
    config::Config,
    meta::MetadataRepo,
    walker::Walker,
};

pub struct App {
    cli: Cli,
    config: Config,
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
            // TODO: Perhaps rename List -> Show, and use List as the command for listing out all
            // the entries, just like `git stash list`
            // Don't forget to update the default impl for `Action`!
            Action::List(list) => list.run(&cmd_data),
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
