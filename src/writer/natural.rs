use crate::parser::ParsedFiles;

pub struct NaturalWriter;

impl NaturalWriter {
    pub fn write_stdout(files: &ParsedFiles) {
        for file in files {
            println!("File {}", &file.file.display());

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
                println!(
                    "  Line {: >ln_width$}: {}",
                    line.line,
                    line.message,
                    ln_width = max_width
                )
            }

            println!();
        }
    }
}
