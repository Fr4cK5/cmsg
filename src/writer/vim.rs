use std::fmt::Write as _;

use crate::parser::ParsedFiles;

pub struct VimFormatter;

impl VimFormatter {
    pub fn format(files: &ParsedFiles) -> String {
        let mut buf = String::new();

        for file in files.0.iter() {
            for line in &file.lines {
                _ = writeln!(
                    &mut buf,
                    "{}:{}={}",
                    file.file.display(),
                    line.line,
                    line.message
                );
            }
        }

        buf
    }
}
