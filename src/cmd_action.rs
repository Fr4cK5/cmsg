use crate::{cli::OutputFormat, config::Config, meta::MetadataRepo, parser::ParsedFiles};

pub mod clean;
pub mod commit;
pub mod count;
pub mod inspect;
pub mod list;
pub mod locate;
pub mod reset;

/// A capability package supplied to every action.
pub struct CmdData<'a> {
    pub files: &'a ParsedFiles,
    pub output: OutputFormat,
    pub config: &'a Config,
    pub repo: &'a MetadataRepo,
}
