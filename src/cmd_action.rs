use crate::{cli::OutputFormat, parser::ParsedFiles};

#[derive(Debug, Clone, Default, clap::Parser)]
pub struct List {
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Copy the output to the system clipboard. Forces compact output format."
    )]
    pub copy: bool,
}

impl List {
    pub fn run(&self, files: &ParsedFiles, output: &OutputFormat) {
        let output = files.to_formatted_string(output);
        println!("{output}");
    }
}

#[derive(Debug, Clone, Default, clap::Parser)]
pub struct Count;

impl Count {
    pub fn run(files: &ParsedFiles, output: &OutputFormat) {
        let file_count = files.0.len();
        let line_count = files.0.iter().map(|item| item.lines.len()).sum::<usize>();

        match output {
            OutputFormat::Natural => {
                println!("Files   : {}\nMessages: {}", file_count, line_count);
            }
            OutputFormat::Vim => {
                println!("files={}\nlines={}", file_count, line_count);
            }
            OutputFormat::Json => {
                println!(r#"{{ "files": {}, "lines": {} }}"#, file_count, line_count)
            }
        }
    }
}
