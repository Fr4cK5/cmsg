// TODO: Unify usages of digest vs hash. I don't care which one you pick but it's important that you
// pick one

use std::time::Instant;

use eyre::Result;

use crate::{app::App, cli::Cli, meta::MetadataRepo};

pub const CREATE_DB: &str = include_str!("../sql/create_db.sql");

mod app;
mod cli;
mod cmd_action;
mod config;
mod hash;
mod meta;
mod parser;
mod walker;
mod writer;

fn main() -> Result<()> {
    let start = Instant::now();

    let cli_args = Cli::parse_and_validate();
    let metadata_repo = MetadataRepo::new()?;
    App::new(cli_args, metadata_repo).run()?;

    let elapsed = start.elapsed();
    dbg!(elapsed);

    Ok(())
}
