//.git-keep

use crate::parser::ParsedFiles;

pub struct VimWriter;

impl VimWriter {
    pub fn write_stdout(files: &ParsedFiles) {
        for file in files {
            for line in &file.lines {
                println!("{}:{}={}", file.file.display(), line.line, line.message);
            }
        }
    }
}
