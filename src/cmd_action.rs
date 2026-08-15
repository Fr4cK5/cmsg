use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use eyre::{Result, eyre};
use rusqlite::named_params;

use crate::{
    cli::OutputFormat,
    config::Config,
    hash,
    meta::{MetadataRepo, types::Backup},
    parser::ParsedFiles,
    pathutil,
    serialize_types::SerializableParsedFile,
};

/// A capability package supplied to every action.
pub struct CmdData<'a> {
    pub files: &'a ParsedFiles,
    pub output: OutputFormat,
    pub config: &'a Config,
    pub repo: &'a MetadataRepo,
}

/// The `ls` subcommand
#[derive(Debug, Clone, Default, clap::Parser)]
pub struct List {
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Copy the output to the system clipboard. Forces vim output format."
    )]
    pub copy: bool,
}

impl List {
    pub fn run(&self, data: &CmdData) -> Result<()> {
        match data.output {
            OutputFormat::Natural => {
                let mut buf = String::new();

                for file in &data.files.0 {
                    _ = writeln!(&mut buf, "File {}", file.file.display());

                    let mut max_len = file
                        .lines
                        .iter()
                        .map(|item| item.line)
                        .max()
                        .unwrap_or_default();

                    let mut max_width = 1usize;
                    while max_len > 10 {
                        max_width += 1;
                        max_len /= 10;
                    }

                    for line in file.lines.iter() {
                        _ = writeln!(
                            &mut buf,
                            "  Line {: >ln_width$}: {}",
                            line.line,
                            line.message,
                            ln_width = max_width
                        );
                    }

                    buf.push('\n');
                }

                println!("{buf}");
            }
            OutputFormat::Vim => {
                let mut buf = String::new();

                for file in &data.files.0 {
                    for line in &file.lines {
                        _ = writeln!(
                            &mut buf,
                            "{}:{}={}",
                            file.file.display(),
                            line.line,
                            line.message
                        );
                    }
                }

                println!("{buf}");
            }
            OutputFormat::Json => {
                let files = data
                    .files
                    .0
                    .iter()
                    .map(|file| SerializableParsedFile::from(file.clone()))
                    .collect::<Vec<_>>();

                let json = serde_json::to_string_pretty(&files)
                    .map_err(|err| eyre!("{err}: Failed to serialize parsed files to json"))?;

                println!("{json}");
            }
        }

        Ok(())
    }
}

/// The `count` subcommand
#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Count;

impl Count {
    pub fn run(data: &CmdData) -> Result<()> {
        let file_count = data.files.0.len();
        let line_count = data
            .files
            .0
            .iter()
            .map(|item| item.lines.len())
            .sum::<usize>();

        match data.output {
            OutputFormat::Natural => {
                println!("Files   : {}\nMessages: {}", file_count, line_count);
            }
            OutputFormat::Vim => {
                println!("files={}\nlines={}", file_count, line_count);
            }
            OutputFormat::Json => {
                println!(r#"{{ "files": {}, "lines": {} }}"#, file_count, line_count)
            }
        }

        Ok(())
    }
}

/// The `commit` subcommand
#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Commit {
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Copy the output to the system clipboard. Forces vim output format."
    )]
    pub copy: bool,
}

impl Commit {
    pub fn run(&self, data: &CmdData) -> Result<()> {
        if data.files.0.is_empty() {
            return Ok(());
        }

        List::default().run(data)?;
        let backup = Self::create_backup(data.files, &data.config.data_directory)?;

        data.repo.transaction(|tx| {
            let data_directory_id = MetadataRepo::upsert_path(tx, &data.config.data_directory)?;
            MetadataRepo::insert_backup_record(tx, &backup, data_directory_id)?;
            Ok(())
        })
    }

    fn create_backup(files: &ParsedFiles, data_dir: &Path) -> Result<Backup> {
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

        Ok(Backup {
            backup_hash,
            data_directory: data_dir.to_owned(),
            backup_root,
        })
    }
}

/// The `clean` subcommand
#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Clean {
    #[arg(
        short = 'a',
        long,
        default_value_t = false,
        help = "Remove *all* data directories instead of just the one's belonging to the local project."
    )]
    pub remove_all: bool,
}

impl Clean {
    pub fn run(&self, data: &CmdData) -> Result<()> {
        if self.remove_all {
            self.remove_all(data)
        } else {
            self.remove_local(data)
        }
    }

    fn fetch_metadata(&self, data: &CmdData) -> Result<Option<(Vec<String>, i64)>> {
        data.repo.transaction(|tx| {
            MetadataRepo::fetch_hashes_for_data_directory(tx, &data.config.data_directory)
        })
    }

    fn remove_local(&self, data: &CmdData) -> Result<()> {
        let (commit_hashes, data_directory_id) = match self.fetch_metadata(data) {
            Ok(None) => return Ok(()),
            Ok(Some((rows, id))) => (rows, id),
            Err(e) => return Err(e),
        };

        data.repo.transaction(|tx| {
            let mut prepared = tx.prepare("delete from backup_entry where hash = :hash")?;
            for commit_hash in &commit_hashes {
                let dir = PathBuf::from(&data.config.data_directory).join(commit_hash);
                if fs::exists(&dir)?
                    && let Err(err) = fs::remove_dir_all(&dir)
                {
                    return Err(eyre!("{err}: Failed to remove entry '{}'", dir.display()));
                }

                prepared.execute(named_params! {
                    ":hash": &commit_hash,
                })?;
            }

            let mut prepared = tx.prepare("delete from data_directory where id = :id")?;
            prepared.execute(named_params! {
                ":id": data_directory_id,
            })?;

            Ok(())
        })?;

        match data.output {
            OutputFormat::Natural => {
                println!("Removed the following entries:");
                for commit_hash in commit_hashes {
                    println!("  {commit_hash}");
                }
            }
            OutputFormat::Vim => {
                for commit_hash in commit_hashes {
                    println!("{commit_hash}");
                }
            }
            OutputFormat::Json => {
                if let Ok(json) = serde_json::to_string_pretty(&commit_hashes) {
                    println!("{json}");
                }
            }
        }

        Ok(())
    }

    fn remove_all(&self, data: &CmdData) -> Result<()> {
        let directories = data.repo.transaction(|tx| {
            let mut prepared = tx.prepare("select path from data_directory")?;
            let rows = prepared.query_map([], |row| row.get::<_, String>(0))?;
            let mut directories = Vec::new();
            for row in rows.flatten() {
                directories.push(row)
            }

            Ok(directories)
        })?;

        data.repo.transaction(|tx| {
            let mut prepared = tx.prepare("delete from data_directory where path = :path")?;
            for dir in &directories {
                if fs::exists(dir).unwrap_or_default()
                    && let Err(e) = fs::remove_dir_all(dir)
                {
                    eprintln!("{e}: Failed to remove data directory '{dir}'");
                } else {
                    prepared.execute(named_params! {
                        ":path": &dir,
                    })?;
                }
            }

            Ok(())
        })?;

        match data.output {
            OutputFormat::Natural => {
                println!("Removed the following data directories:");
                for dir in directories {
                    println!("  {dir}");
                }
            }
            OutputFormat::Vim => {
                for dir in directories {
                    println!("{dir}");
                }
            }
            OutputFormat::Json => {
                if let Ok(json) = serde_json::to_string_pretty(&directories) {
                    println!("{json}");
                }
            }
        }

        Ok(())
    }
}
