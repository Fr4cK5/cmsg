use std::{
    fmt::Display,
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use eyre::Result;
use ignore::{WalkBuilder, WalkParallel, WalkState};

use crate::{
    cli::Cli,
    hash,
    parser::{ParsedFile, ParsedFiles, Parser},
};

/// A wrapper around the `ignore::WalkParallel` type with additional information.
///
/// This wrapper currently exclusively uses the parallel walker, as the intended use case /
/// environment for cmsg is on a "modern" multi-core CPU system.
pub struct Walker {
    /// The walk base, so from where the `WalkParallel` starts walking the directory hierarchy.
    walk_base: PathBuf,

    /// The built walker implementation.
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
        let start = Instant::now();

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

        // performance metrics
        let stats_total_files = Arc::new(AtomicUsize::new(0));
        let stats_matched_files = Arc::new(AtomicUsize::new(0));
        let stats_total_read_bytes = Arc::new(AtomicUsize::new(0));

        thread::scope(|scope| {
            let threads: usize = thread::available_parallelism().map_or(4usize, |nz| nz.into());
            let mut senders = Vec::<Sender<PathBuf>>::with_capacity(threads);

            let (join_sender, join_receiver) = mpsc::channel::<ParsedFile>();

            for _ in 0..threads {
                let stats_total_files = stats_total_files.clone();
                let stats_matched_files = stats_matched_files.clone();
                let stats_total_read_bytes = stats_total_read_bytes.clone();

                let walk_base = self.walk_base.clone();

                // Into parser
                let (in_sender, in_receiver) = mpsc::channel::<PathBuf>();

                // Out from parser
                let (out_sender, out_receiver) = mpsc::channel::<ParsedFile>();

                // Collect all in the end
                let joiner = join_sender.clone();

                senders.push(in_sender);

                scope.spawn(move || {
                    let mut total_files = 0;
                    let mut matched_files = 0;
                    let mut total_read_bytes = 0;

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

                        total_files += 1;
                        total_read_bytes += content.len();

                        if !result.is_empty()
                            && let Some(path) = pathdiff::diff_paths(&file_name, &walk_base)
                        {
                            matched_files += 1;
                            out_sender
                                .send(ParsedFile::new(file_name, result, file_hash, path, content))
                                .ok();
                        }
                    }

                    stats_total_files.fetch_add(total_files, Ordering::Release);
                    stats_matched_files.fetch_add(matched_files, Ordering::Release);
                    stats_total_read_bytes.fetch_add(total_read_bytes, Ordering::Release);
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

            let duration = start.elapsed();

            let stats = WalkStats {
                total_files: stats_total_files.load(Ordering::Acquire),
                matched_files: stats_matched_files.load(Ordering::Acquire),
                bytes_read: stats_total_read_bytes.load(Ordering::Acquire),
                duration,
            };

            ParsedFiles {
                files: parsed_files,
                stats,
            }
        })
    }
}

/// A Simple struct that holds all of our performance mentrics data to be able to uniformly pass
/// them around and display them in text-form.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WalkStats {
    pub total_files: usize,
    pub matched_files: usize,
    pub bytes_read: usize,
    pub duration: Duration,
}

impl Display for WalkStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration_secs = self.duration.as_secs_f64();
        let overall_size = format_data_size(self.bytes_read);

        f.write_str("========== STATS ==========\n\n")?;
        f.write_str("Base stats:\n")?;
        f.write_fmt(format_args!(
            "  Total data read: {:.03}{}\n",
            overall_size.0, overall_size.1,
        ))?;

        f.write_fmt(format_args!("  Total files read: {}\n", self.total_files))?;
        f.write_fmt(format_args!(
            "  Files with matches: {}\n",
            self.matched_files
        ))?;
        f.write_fmt(format_args!("  Walking + Parsing: {:?}\n", self.duration))?;

        let per_second_size = format_data_size((self.bytes_read as f64 / duration_secs) as usize);

        f.write_str("\nThroughput:\n")?;
        f.write_fmt(format_args!(
            "  Data: {:.03}{}/s\n",
            per_second_size.0, per_second_size.1
        ))?;
        f.write_fmt(format_args!(
            "  Files: {:.03}/s",
            self.total_files as f64 / duration_secs
        ))?;

        f.write_str("\n\n========== STATS ==========")
    }
}

fn format_data_size(size: usize) -> (f64, String) {
    let mut value = size as f64;
    let mut unit = 0;

    while value >= 1000.0 {
        value /= 1000.0;
        unit += 1;
    }

    let suffix = match unit {
        0 => "B",
        1 => "KB",
        2 => "MB",
        3 => "GB",
        4 => "TB",
        5 => "PB",
        6 => "EB",
        7 => "ZB",
        8 => "YB",
        9 => "RB",
        10 => "QB",
        n => &format!("*1E{} (non-si-unit)", n * 3),
    };

    (value, suffix.to_owned())
}
