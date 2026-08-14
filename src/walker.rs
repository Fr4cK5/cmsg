use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{self, Sender},
    thread,
};

use eyre::Result;
use ignore::{WalkBuilder, WalkParallel, WalkState};

use crate::{
    cli::Cli,
    hash,
    parser::{ParsedFile, ParsedFiles, Parser},
};

pub struct Walker {
    walk_base: PathBuf,
    walker: WalkParallel,
}

impl Walker {
    pub fn build_from(cli: &Cli) -> Result<Self> {
        let walk_base = PathBuf::from(&cli.base_directory).canonicalize()?;

        let walker = WalkBuilder::new(&walk_base)
            .ignore(!cli.no_ignore)
            .git_ignore(!cli.no_ignore)
            .git_global(!cli.no_ignore)
            .hidden(!cli.include_hidden)
            .add_custom_ignore_filename(".rgignore")
            .add_custom_ignore_filename(".fdignore")
            .build_parallel();

        Ok(Self { walk_base, walker })
    }

    pub fn walk(self) -> ParsedFiles {
        let (sender, receiver) = mpsc::channel::<PathBuf>();

        self.walker.run(|| {
            let sender = sender.clone();
            Box::new(move |entry| {
                let Ok(entry) = entry else {
                    return WalkState::Skip;
                };

                if entry.file_type().map(|ft| ft.is_file()).unwrap_or_default() {
                    sender.send(entry.into_path()).ok();
                }

                WalkState::Continue
            })
        });

        // Drop the sender to make sure the receivers don't deadlock, even after all the cloned
        // senders are dropped.
        drop(sender);

        thread::scope(|scope| {
            let threads: usize = thread::available_parallelism().map_or(4usize, |nz| nz.into());
            let mut senders = Vec::<Sender<PathBuf>>::with_capacity(threads);

            let (join_sender, join_receiver) = mpsc::channel::<ParsedFile>();

            for _ in 0..threads {
                // TODO: Use this with the pathdiff crate. Apparently it lets us do the thing we
                // need to, to get a path relative to the walk_base instead of an absolute one.
                let walk_base = self.walk_base.clone();

                // Into parser
                let (in_sender, in_receiver) = mpsc::channel::<PathBuf>();

                // Out from parser
                let (out_sender, out_receiver) = mpsc::channel::<ParsedFile>();

                // Collect all in the end
                let joiner = join_sender.clone();

                senders.push(in_sender);

                scope.spawn(move || {
                    while let Ok(path) = in_receiver.recv() {
                        let Ok(content) = fs::read_to_string(&path) else {
                            // Any errors coming from here MUST be errors related to trying to read
                            // non-UTF8 data into a string. Rust `String`s always contain valid
                            // UTF8, meaning that a failure simply means we've hit a binary file.
                            continue;
                        };

                        let file_name = path.into_os_string();
                        let file_hash = hash::sha256_hash_alloc(content.as_bytes());
                        let mut parser = Parser::new(&content);
                        let result = parser.parse();

                        if !result.is_empty()
                            && let Some(path) = pathdiff::diff_paths(&file_name, &walk_base)
                        {
                            out_sender
                                .send(ParsedFile::new(file_name, result, file_hash, path))
                                .ok();
                        }
                    }
                });

                scope.spawn(move || {
                    while let Ok(parsed_file) = out_receiver.recv() {
                        joiner.send(parsed_file).ok();
                    }
                });
            }

            drop(join_sender);

            let mut current_item_index = 0usize;
            while let Ok(path) = receiver.recv() {
                let sender = senders.get(current_item_index % senders.len());
                if let Some(sender) = sender {
                    sender.send(path).ok();
                }
                current_item_index += 1;
            }

            drop(senders);

            let mut parsed_files = Vec::new();
            while let Ok(parsed_file) = join_receiver.recv() {
                parsed_files.push(parsed_file);
            }

            ParsedFiles(parsed_files)
        })
    }
}
