use std::fmt::Write as _;

use crate::parser::ParsedFiles;

pub struct NaturalFormatter;

impl NaturalFormatter {
    pub fn format(files: &ParsedFiles) -> String {
        let mut buf = String::new();

        for file in files.0.iter() {
            _ = writeln!(&mut buf, "File {}", &file.file.display());

            let mut max_len = file
                .lines
                .iter()
                .map(|item| item.line)
                .max()
                .unwrap_or_default();

            let mut max_width = 1usize;
            while max_len > 10 {
                max_width += 1;
                max_len /= 10;
            }

            for line in file.lines.iter() {
                _ = writeln!(
                    &mut buf,
                    "  Line {: >ln_width$}: {}",
                    line.line,
                    line.message,
                    ln_width = max_width
                );
            }

            buf.push('\n');
        }

        buf
    }
}
