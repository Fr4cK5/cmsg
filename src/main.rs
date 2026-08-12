use std::{path::PathBuf, time::Instant};

use eyre::Result;

use crate::{
    app::App,
    cli::Cli,
    config::{Config, StorageStrategy},
    meta::MetadataRepo,
};

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

    let config = Config::load(
        &PathBuf::from(&cli_args.base_directory),
        StorageStrategy::default(),
    )
    .ok()
    .unwrap_or_default();

    let metadata_repo = MetadataRepo::new(&config.data_directory)?;

    App::new(cli_args, config, metadata_repo).run()?;

    let elapsed = start.elapsed();
    dbg!(elapsed);

    Ok(())
}
