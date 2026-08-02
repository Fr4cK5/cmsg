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
        output.write_stdout(files);
    }
}
