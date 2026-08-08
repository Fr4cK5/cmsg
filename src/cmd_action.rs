use std::{
    fs,
    path::{Path, PathBuf},
};

use eyre::{Result, eyre};

use crate::{
    cli::OutputFormat,
    config::Config,
    hash,
    meta::{MetadataRepo, types::Backup},
    parser::ParsedFiles,
};

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
    pub fn run(&self, files: &ParsedFiles, output: &OutputFormat) -> Result<()> {
        let output = if self.copy {
            files.to_formatted_string(&OutputFormat::Vim)
        } else {
            files.to_formatted_string(output)
        };

        match &output {
            Ok(output) => println!("{output}"),
            Err(err) => eprintln!("{err}"),
        }

        output.map(|_| ())
    }
}

#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Count;

impl Count {
    pub fn run(files: &ParsedFiles, output: &OutputFormat) -> Result<()> {
        let file_count = files.0.len();
        let line_count = files.0.iter().map(|item| item.lines.len()).sum::<usize>();

        match output {
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
    pub fn run(
        &self,
        files: &ParsedFiles,
        output: &OutputFormat,
        config: &Config,
        repo: &MetadataRepo,
    ) -> Result<()> {
        if files.0.is_empty() {
            return Ok(());
        }

        List::default().run(files, output)?;
        let backup = Self::create_backup(files, &config.data_directory)?;

        let mut conn = repo.connection.borrow_mut();
        let tx = conn.transaction()?;
        let data_directory_id = MetadataRepo::upsert_path(&tx, &config.data_directory)?;
        MetadataRepo::insert_backup_record(&tx, &backup, data_directory_id)?;

        Ok(tx.commit()?)
    }

    fn create_backup(files: &ParsedFiles, data_dir: &Path) -> Result<Backup> {
        let mut buf = [0u8; 256];
        fastrand::fill(&mut buf);
        let digest = hash::sha256_digest_alloc(&buf);
        let backup_root = data_dir.join(&digest);

        fs::create_dir_all(&backup_root).map_err(|err| {
            eyre!("{err}: Failed to create storage directory, your files have not been touched.")
        })?;

        for file in &files.0 {
            let source = file.file.as_os_str();
            let destination = backup_root.join(source);
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
            digest,
            data_directory: data_dir.to_owned(),
            backup_root,
        })
    }
}

#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Clean {
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Remove *all* data directories instead of just the one's belonging to the local project."
    )]
    pub all: bool,
}
