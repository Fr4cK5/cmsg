use std::fs;

use eyre::Result;

use crate::{
    cli::{Action, Cli},
    cmd_action::Count,
    config::{Config, StorageStrategy},
    meta::MetadataRepo,
    walker::Walker,
};

pub struct App {
    cli: Cli,
    config: Config,
    metadata_repo: MetadataRepo,
}

impl App {
    pub fn new(cli: Cli, metadata_repo: MetadataRepo) -> Self {
        Self {
            cli,
            config: Config::load(StorageStrategy::default())
                .ok()
                .unwrap_or_default(),
            metadata_repo,
        }
    }

    /// Run the action
    pub fn run(&self) -> Result<()> {
        let walker = Walker::build_from(&self.cli);
        let mut parsed_files = walker.walk();
        parsed_files.sort();

        match self.cli.action.as_ref().unwrap_or(&Action::default()) {
            Action::List(list) => list.run(&parsed_files, &self.cli.format),
            Action::Commit(commit) => commit.run(
                &parsed_files,
                &self.cli.format,
                &self.config,
                &self.metadata_repo,
            ),
            Action::Undo => todo!(),
            Action::Count => Count::run(&parsed_files, &self.cli.format),
            Action::Clean(_clean) => {
                // TODO: Default to only removing the project-local data dirs.
                // Only remove all the data dirs if `all` is true
                // This means we'll have to keep some metadata about the hashes and such.
                let mut errors = false;

                for dir in [
                    StorageStrategy::dotgit_data(),
                    StorageStrategy::user_home_data(),
                ] {
                    if let Some(data) = dir
                        && fs::exists(&data)?
                        && let Err(e) = fs::remove_dir_all(&data)
                    {
                        eprintln!("{e}: Failed to remove project-local '{}'", data.display());
                        errors = true;
                    }
                }

                if errors {
                    eprintln!(
                        "If the reported error is about not being able to find the specified directory you can safely ignore it.\nOtherwise something went wrong."
                    );
                }

                Ok(())
            }
        }?;

        Ok(())
    }
}
