use std::{fs, path::PathBuf};

use eyre::{Result, eyre};

use crate::{cli::OutputFormat, config::Config, hash, parser::ParsedFiles};

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
    pub fn run(&self, files: &ParsedFiles, output: &OutputFormat) {
        let output = if self.copy {
            files.to_formatted_string(&OutputFormat::Vim)
        } else {
            files.to_formatted_string(output)
        };

        println!("{output}");
    }
}

#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Count;

impl Count {
    pub fn run(files: &ParsedFiles, output: &OutputFormat) {
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
    copy: bool,
}

impl Commit {
    pub fn run(&self, files: &ParsedFiles, config: &Config) -> Result<()> {
        List::default().run(files, &OutputFormat::Vim);

        let mut buf: [u8; 128] = [0u8; 128];
        fastrand::fill(&mut buf);
        let digest = hash::sha256_digest_alloc(&buf);
        let storage_dir = config.data_directory.join(digest);

        fs::create_dir_all(&storage_dir).map_err(|err| {
            eyre!("{err}: Failed to create storage directory, your files have not been touched.")
        })?;

        for file in &files.0 {
            let source = file.file.as_os_str();
            let destination = storage_dir.join(source);
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

        Ok(())
    }
}
