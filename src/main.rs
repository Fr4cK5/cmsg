#![feature(normalize_lexically)]

#[cfg(debug_assertions)]
use std::time::Instant;

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
mod fs_ext;
mod hash;
mod meta;
mod parser;
mod pathutil;
mod serialize_types;
mod trie;
mod walker;

fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    let start = Instant::now();

    let cli_args = Cli::parse_and_validate();

    let config = Config::load(
        &pathutil::normalize(&cli_args.base_directory)?,
        StorageStrategy::default(),
    )
    .ok()
    .unwrap_or_default();

    let metadata_repo = MetadataRepo::new()?;

    if cfg!(debug_assertions) {
        App::new(cli_args, config, metadata_repo).run()?;
    } else if let Err(err) = App::new(cli_args, config, metadata_repo).run() {
        eprintln!("Error: {err}");
    }

    #[cfg(debug_assertions)]
    {
        let elapsed = start.elapsed();
        dbg!(elapsed);
    }

    Ok(())
}
