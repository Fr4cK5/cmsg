use std::{
    fmt::Display,
    fs,
    ops::{Add, AddAssign},
    path::PathBuf,
    sync::mpsc::{self, Sender},
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

        thread::scope(|scope| {
            let threads: usize = thread::available_parallelism().map_or(4usize, |nz| nz.into());
            let mut senders = Vec::<Sender<PathBuf>>::with_capacity(threads);

            let (join_sender, join_receiver) = mpsc::channel::<ParsedFile>();
            let mut pool = Vec::new();

            for _ in 0..threads {
                let walk_base = self.walk_base.clone();

                // Into parser
                let (in_sender, in_receiver) = mpsc::channel::<PathBuf>();

                // Out from parser
                let (out_sender, out_receiver) = mpsc::channel::<ParsedFile>();

                // Collect all in the end
                let joiner = join_sender.clone();

                senders.push(in_sender);

                let handle = scope.spawn(move || {
                    let mut stats = WalkStats::default();

                    while let Ok(path) = in_receiver.recv() {
                        let read_start = Instant::now();
                        let Ok(content) = fs::read_to_string(&path) else {
                            // Any errors coming from here MUST be errors related to trying to read
                            // non-UTF8 data into a string. Rust `String`s always contain valid
                            // UTF8, meaning that a failure simply means we've hit a binary file.
                            stats.file_read_duration += read_start.elapsed();
                            continue;
                        };
                        stats.file_read_duration += read_start.elapsed();

                        let file_name = path.into_os_string();
                        let file_hash = hash::sha256_hash_alloc(content.as_bytes());
                        let mut parser = Parser::new(&content);
                        let parse_start = Instant::now();
                        let result = parser.parse();
                        stats.parse_duration += parse_start.elapsed();

                        stats.total_files += 1;
                        stats.bytes_read += content.len();

                        if !result.is_empty()
                            && let Some(path) = pathdiff::diff_paths(&file_name, &walk_base)
                        {
                            stats.matched_files += 1;
                            out_sender
                                .send(ParsedFile::new(file_name, result, file_hash, path, content))
                                .ok();
                        }
                    }

                    stats
                });

                pool.push(handle);

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

            let full_duration = start.elapsed();

            let mut stats = WalkStats {
                full_duration,
                threads_used: threads,
                ..Default::default()
            };

            for handle in pool {
                stats += handle.join().unwrap_or_default();
            }

            ParsedFiles {
                files: parsed_files,
                stats,
            }
        })
    }
}

/// A Simple struct that holds all of our performance metrics data to be able to uniformly pass
/// them around and display them in text-form.
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct WalkStats {
    pub total_files: usize,
    pub matched_files: usize,
    pub bytes_read: usize,
    pub threads_used: usize,
    pub file_read_duration: Duration,
    pub parse_duration: Duration,
    pub full_duration: Duration,
}

impl Add for WalkStats {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            total_files: self.total_files + rhs.total_files,
            matched_files: self.matched_files + rhs.matched_files,
            bytes_read: self.bytes_read + rhs.bytes_read,
            threads_used: self.threads_used + rhs.threads_used,
            file_read_duration: self.file_read_duration + rhs.file_read_duration,
            parse_duration: self.parse_duration + rhs.parse_duration,
            full_duration: self.full_duration + rhs.full_duration,
        }
    }
}

impl AddAssign for WalkStats {
    fn add_assign(&mut self, rhs: Self) {
        self.total_files += rhs.total_files;
        self.matched_files += rhs.matched_files;
        self.bytes_read += rhs.bytes_read;
        self.threads_used += rhs.threads_used;
        self.file_read_duration += rhs.file_read_duration;
        self.parse_duration += rhs.parse_duration;
        self.full_duration += rhs.full_duration;
    }
}

impl Display for WalkStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration_secs = self.full_duration.as_secs_f64();
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
        f.write_fmt(format_args!("  Full pipeline: {:?}\n", self.full_duration))?;
        f.write_fmt(format_args!(
            "  (CPU) Time spent reading files: {:?}\n",
            self.file_read_duration
        ))?;
        f.write_fmt(format_args!(
            "  (CPU) Time spent parsing: {:?}\n",
            self.parse_duration
        ))?;
        f.write_fmt(format_args!(
            "  Thread pool: {} core(s)\n",
            self.threads_used
        ))?;

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
