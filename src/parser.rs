use std::{
    ffi::OsString,
    num::{NonZeroU64, NonZeroUsize},
    path::PathBuf,
    simd::{Simd, cmp::SimdPartialEq},
};

use serde::Serialize;

use crate::walker::WalkStats;

pub const MARKER: &str = ".cmsg";
pub const SIMD_LANES: usize = 64;

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
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(value: &'a str) -> Self {
        Self {
            input: value,
            pos: 0,
        }
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

    pub fn simd_parse(&mut self) -> Vec<ParsedLine> {
        let mut output = Vec::new();
        while self.input.len() <= self.pos + SIMD_LANES {
            if let Some(bits) = unsafe { self.simd_find_byte(b'.') } {
                let mut mask = bits.get();
                while mask != 0u64 {
                    let idx = self.pos + mask.trailing_zeros() as usize;
                    if let Some(result) = self.try_parse_from_offset(idx).map(ToOwned::to_owned) {
                        output.push(ParsedLine::new(0, result));
                    }
                    mask &= mask - 1;
                }
            }
            self.pos += SIMD_LANES;
        }

        // self.scan_remaining()

        output
    }

    unsafe fn simd_find_byte(&mut self, byte: u8) -> Option<NonZeroU64> {
        let base = Simd::<u8, SIMD_LANES>::splat(byte);
        let values = Simd::from_slice(&self.input.as_bytes()[self.pos..self.pos + SIMD_LANES]);
        let mask = base.simd_eq(values);
        let bits = mask.to_bitmask();
        NonZeroU64::new(bits)
    }

    fn try_parse_from_offset(&self, offset: usize) -> Option<&str> {
        let start = self.pos + offset;
        self.input.get(start..start + MARKER.len() + 1)
    }
}
