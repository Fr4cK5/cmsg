use std::{ffi::OsString, path::PathBuf};

use serde::Serialize;

use crate::walker::WalkStats;

pub const MARKER: &str = ".cmsg";

/// ParsedLine represents a single, context-less, parsed line.
/// It consists of a line number where its .cmsg was found, and the contained message.
///
/// Note: The line number is one-indexed. Iterators you'd get from doing
/// `file_content.lines().enumerate()` or similar, use zero-indexed ranges.
#[derive(Debug, Clone, Serialize)]
pub struct ParsedLine {
    pub line: usize,
    pub message: String,
}

impl ParsedLine {
    pub fn new(line: usize, message: String) -> Self {
        Self { line, message }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ParsedFiles {
    pub files: Vec<ParsedFile>,
    pub stats: WalkStats,
}

impl ParsedFiles {
    pub fn sort(&mut self) {
        self.files.sort_by(|a, b| a.file.cmp(&b.file));
    }
}

/// ParsedFile represents a whole file of parsed lines.
#[derive(Debug, Clone, Serialize)]
pub struct ParsedFile {
    /// The full file path
    pub file: OsString,

    /// The file path, relative to the base directory (`-d` flag)
    pub relative_path: PathBuf,

    /// Parsed .cmsg lines contained withing this file's content
    pub lines: Vec<ParsedLine>,

    /// A Sha-256 hash of the file's contents
    pub hash: String,

    /// The file's content
    pub content: String,
}

impl ParsedFile {
    pub fn new(
        file: OsString,
        lines: Vec<ParsedLine>,
        hash: String,
        relative_path: PathBuf,
        content: String,
    ) -> Self {
        Self {
            file,
            lines,
            hash,
            relative_path,
            content,
        }
    }
}

/// Parser to find .cmsg markers, their line number and their content.
pub struct Parser<'a> {
    input: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(value: &'a str) -> Self {
        Self { input: value }
    }

    pub fn parse(&mut self) -> Vec<ParsedLine> {
        self.input
            .lines()
            .enumerate()
            .flat_map(|(idx, line)| {
                let line_number = idx + 1;
                let message_start = line.find(MARKER)? + MARKER.len() + 1;
                let message = line.get(message_start..).map(String::from)?;
                Some(ParsedLine::new(line_number, message))
            })
            .collect::<Vec<_>>()
    }
}
