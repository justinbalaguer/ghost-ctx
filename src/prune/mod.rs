//! Tree-sitter skeleton engine: keeps signatures/types, collapses bodies.
mod lang;
mod skeleton;
#[cfg(test)]
mod tests;

use std::path::Path;

pub use skeleton::skeleton;

/// True when `ext` (no dot) has a grammar.
pub fn is_supported(ext: &str) -> bool {
    lang::for_ext(ext).is_some()
}

pub fn run(file: &Path) -> anyhow::Result<()> {
    let src = std::fs::read_to_string(file)?;
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
    let out = skeleton(&src, ext).ok_or_else(|| anyhow::anyhow!("unsupported file type: {ext:?}"))?;
    print!("{out}");
    Ok(())
}
