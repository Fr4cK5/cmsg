use serde::Serialize;

use crate::parser::{ParsedFile, ParsedLine};

/// A JSON-Formattable parsed file, used for json formatted output of the `ls` command.
#[derive(Debug, Serialize)]
pub struct SerializableParsedFile {
    file: String,
    lines: Vec<ParsedLine>,
}

impl From<ParsedFile> for SerializableParsedFile {
    fn from(value: ParsedFile) -> Self {
        Self {
            file: value.file.to_string_lossy().to_string(),
            lines: value.lines,
        }
    }
}
