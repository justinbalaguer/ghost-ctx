//! Secret detection (regex + entropy) and deterministic redaction.
mod detect;
mod redactor;
mod scan;
#[cfg(test)]
mod tests;

pub use redactor::Redactor;
pub use scan::scan;
