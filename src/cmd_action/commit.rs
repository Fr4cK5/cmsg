use std::{
    fs,
    path::{Path, PathBuf},
};

use eyre::{ContextCompat, Result, eyre};

use crate::{
    cli::OutputFormat,
    cmd_action::{CmdData, list::List},
    hash,
    meta::{MetadataRepo, types::CommitData},
    parser::ParsedFiles,
    pathutil,
    trie::PrefixTrie,
};

/// The `commit` subcommand
#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Commit {
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Copy the output to the system clipboard"
    )]
    pub copy: bool,

    #[arg(
        short = 'L',
        long,
        default_value_t = false,
        help = "Don't list the removed .cmsg markers"
    )]
    pub no_list: bool,

    #[arg(
        short = 'S',
        long,
        default_value_t = false,
        help = "Print the full 64 chars long hash instead of just the smaller version that already makes it unique"
    )]
    pub no_short_hash: bool,
}

impl Commit {
    pub fn run(&self, data: &CmdData) -> Result<()> {
        // TODO: Actually remove the cmsg markers from the code, but definitely implement
        // the reset functionality first.
        if data.files.0.is_empty() {
            return Ok(());
        }

        if !self.no_list {
            List::default().run(data)?;
        }
        let commit = Self::create_commit(data.files, &data.config.data_directory)?;

        data.repo.transaction(|tx| {
            let data_directory_id = MetadataRepo::upsert_path(tx, &data.config.data_directory)?;
            MetadataRepo::insert_backup_record(tx, &commit, data_directory_id)?;
            Ok(())
        })?;

        let commit_hash = if self.no_short_hash {
            &commit.hash
        } else {
            let hashes = data.repo.transaction(|tx| {
                MetadataRepo::fetch_backup_hashes(tx, &data.config.data_directory)
            })?;

            let trie = PrefixTrie::from(hashes);
            let (len, path) = trie
                .get_shortest_unique_path(commit.hash)
                .wrap_err(eyre!("This should never happen"))?;

            &path.clone()[0..len]
        };

        match data.output {
            OutputFormat::Natural => println!("Commit hash: {}", commit_hash),
            OutputFormat::Vim => println!("{}", commit_hash),
            OutputFormat::Json => println!(r#""{}""#, commit_hash),
        }

        Ok(())
    }

    fn create_commit(files: &ParsedFiles, data_dir: &Path) -> Result<CommitData> {
        let mut buf = [0u8; 256];
        fastrand::fill(&mut buf);
        let backup_hash = hash::sha256_hash_alloc(&buf);
        let backup_root = data_dir.join(&backup_hash);

        fs::create_dir_all(&backup_root).map_err(|err| {
            eyre!("{err}: Failed to create storage directory, your files have not been touched.")
        })?;

        for file in &files.0 {
            let source = file.file.as_os_str();
            let destination = pathutil::normalize(backup_root.join(&file.relative_path))?;
            let destination_parent = destination
                .parent()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| PathBuf::from("."));

            fs::create_dir_all(&destination_parent).map_err(|err| {
                eyre!(
                    "{err}: Unable to create nested directory to backup the file '{}'",
                    source.display(),
                )
            })?;

            fs::copy(source, &destination).map_err(|err| {
                eyre!(
                    "{err}: Failed to copy file '{}' to '{}'",
                    source.display(),
                    destination.display(),
                )
            })?;
        }

        Ok(CommitData {
            hash: backup_hash,
            data_directory: data_dir.to_owned(),
            backup_root,
        })
    }
}
