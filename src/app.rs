use std::fs;

use eyre::Result;

use crate::{
    cli::{Action, Cli},
    cmd_action::Count,
    config::{Config, StorageStrategy},
    walker::Walker,
};

pub struct App {
    cli: Cli,
    config: Config,
}

impl App {
    pub fn new(cli: Cli) -> Self {
        Self {
            cli,
            config: Config::load(StorageStrategy::GlobalFallback)
                .ok()
                .unwrap_or_default(),
        }
    }

    /// Run the action
    pub fn run(&self) -> Result<()> {
        let walker = Walker::build_from(&self.cli);
        let mut parsed_files = walker.walk();
        parsed_files.sort();

        match self.cli.action.as_ref().unwrap_or(&Action::default()) {
            Action::List(list) => list.run(&parsed_files, &self.cli.format),
            Action::Commit(commit) => commit.run(&parsed_files, &self.config),
            Action::Undo => todo!(),
            Action::Count => Count::run(&parsed_files, &self.cli.format),
            Action::Clean => {
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
