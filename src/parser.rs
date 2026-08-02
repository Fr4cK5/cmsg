use std::ffi::OsString;

const MARKER: &str = ".cmsg";

/// ParsedLine represents a single, context-less, parsed line.
/// It consists of a line number where its .cmsg was found, and the contained message.
///
/// Note: The line number is one-indexed. Iterators you'd get from doing
/// `file_content.lines().enumerate()` or similar, use zero-indexed ranges.
#[derive(Debug, Clone)]
pub struct ParsedLine {
    pub line: usize,
    pub message: String,
}

impl ParsedLine {
    pub fn new(line: usize, message: String) -> Self {
        Self { line, message }
    }
}

pub type ParsedFiles = Vec<ParsedFile>;

/// ParsedFile represents a whole file of parsed lines.
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub file: OsString,
    pub lines: Vec<ParsedLine>,
}

impl ParsedFile {
    pub fn new(file: OsString, lines: Vec<ParsedLine>) -> Self {
        Self { file, lines }
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
