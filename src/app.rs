use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{self, Sender},
    thread,
    time::Instant,
};

use eyre::Result;
use ignore::{WalkBuilder, WalkParallel, WalkState};

use crate::{
    cli::{Action, Cli},
    parser::{ParsedFile, Parser},
};

pub struct App {
    cli: Cli,
}

impl App {
    pub fn new(cli: Cli) -> Self {
        Self { cli }
    }

    // TODO: clean this function up a bit. or a lot. A lot, yeah.

    /// Run the action
    pub fn run(&self) -> Result<()> {
        let (sender, receiver) = mpsc::channel::<PathBuf>();

        self.build_walker().run(|| {
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

        let start = Instant::now();

        let parsed_files = thread::scope(|scope| {
            let threads: usize = thread::available_parallelism().map_or(4usize, |nz| nz.into());
            let mut senders = Vec::<Sender<PathBuf>>::with_capacity(threads);

            let (join_sender, join_receiver) = mpsc::channel::<ParsedFile>();

            for _ in 0..threads {
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

                        let mut parser = Parser::new(&content);
                        let result = parser.parse();

                        if !result.is_empty() {
                            out_sender.send(ParsedFile::new(file_name, result)).ok();
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

            parsed_files
        });

        let elapsed = start.elapsed();
        dbg!(parsed_files, elapsed);

        // TODO: When refactoring this, move the current behavior to the List variant.
        // (probably) create individual types per variant (subcommand) and implement the actual
        // logic on those types's `run` methods
        match self.cli.action.as_ref() {
            None => panic!(),
            Some(Action::List) => todo!(),
            Some(Action::Commit) => todo!(),
            Some(Action::Undo) => todo!(),
            Some(Action::Count) => println!("Yo, {}", 420),
        }

        Ok(())
    }

    fn build_walker(&self) -> WalkParallel {
        WalkBuilder::new(&self.cli.base_directory)
            .ignore(!self.cli.no_ignore)
            .git_ignore(!self.cli.no_ignore)
            .git_global(!self.cli.no_ignore)
            .hidden(!self.cli.include_hidden)
            .add_custom_ignore_filename(".rgignore")
            .add_custom_ignore_filename(".fdignore")
            .build_parallel()
    }
}
