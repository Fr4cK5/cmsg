use eyre::{Result, eyre};
use serde::Serialize;

use crate::parser::{ParsedFile, ParsedFiles, ParsedLine};

pub struct JsonFormatter;

#[derive(Debug, Serialize)]
struct SerializableParsedFile {
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

impl JsonFormatter {
    pub fn format(files: &ParsedFiles) -> Result<String> {
        let files = files
            .0
            .iter()
            .map(|file| SerializableParsedFile::from(file.clone()))
            .collect::<Vec<_>>();

        serde_json::to_string_pretty(&files)
            .map_err(|err| eyre!("{err}: Failed to serialize parsed files to json"))
    }
}
