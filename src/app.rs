use eyre::Result;

use crate::{
    cli::{Action, Cli},
    cmd_action::Count,
    walker::Walker,
};

pub struct App {
    cli: Cli,
}

impl App {
    pub fn new(cli: Cli) -> Self {
        Self { cli }
    }

    /// Run the action
    pub fn run(&self) -> Result<()> {
        let walker = Walker::build_from(&self.cli);
        let mut parsed_files = walker.walk();
        parsed_files.sort();

        match self.cli.action.as_ref().unwrap_or(&Action::default()) {
            Action::List(list) => {
                list.run(&parsed_files, &self.cli.format);
            }
            Action::Commit => todo!(),
            Action::Undo => todo!(),
            Action::Count => Count::run(&parsed_files, &self.cli.format),
        }

        Ok(())
    }
}
