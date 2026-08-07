use std::time::Instant;

use eyre::Result;

use crate::{app::App, cli::Cli};

mod app;
mod cli;
mod cmd_action;
mod config;
mod hash;
mod parser;
mod walker;
mod writer;

fn main() -> Result<()> {
    let cli_args = Cli::parse_and_validate();

    let start = Instant::now();

    App::new(cli_args).run()?;

    let elapsed = start.elapsed();
    dbg!(elapsed);

    Ok(())
}
