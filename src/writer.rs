// TODO: So, if we keep the current idea of the OutputFormat being the "middleware" for outputting
// text, we might wanna make an individual formatter for every action,
// (eg. `NaturalListFormatter`, `VimListFormatter`, `JsonListFormatter`), which then all impl some
// `OutputFormatter` trait that either immediately prints or returns a formatted string. Not sure.
// These would the reside in their respective module below

pub mod json;
pub mod natural;
pub mod vim;
