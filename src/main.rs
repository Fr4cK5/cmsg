#![feature(normalize_lexically)]

use eyre::Result;

use crate::{app::App, cli::Cli};

mod app;
mod cli;
mod parser;

fn main() -> Result<()> {
    let cli_args = Cli::parse_and_validate();

    App::new(cli_args).run()?;

    Ok(())
}
