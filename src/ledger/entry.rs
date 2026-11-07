use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub timestamp: u64,
    pub session_id: String,
    /// "read" (skeleton) or "output" (condensed command output)
    pub kind: String,
    pub file_path: String,
    pub original_token_estimate: usize,
    pub skeleton_token_estimate: usize,
    pub tokens_saved: usize,
}

impl Entry {
    pub fn new(session_id: &str, kind: &str, file_path: &str, original: usize, kept: usize) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs()),
            session_id: session_id.into(),
            kind: kind.into(),
            file_path: file_path.into(),
            original_token_estimate: original,
            skeleton_token_estimate: kept,
            tokens_saved: original.saturating_sub(kept),
        }
    }
}

/// `~/.ghost-ctx`, overridable with GHOST_CTX_HOME (tests).
pub fn home() -> PathBuf {
    std::env::var_os("GHOST_CTX_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".ghost-ctx")))
        .unwrap_or_else(|| PathBuf::from(".ghost-ctx"))
}

pub fn append_to(dir: &std::path::Path, e: &Entry) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("ledger.jsonl"))?;
    writeln!(f, "{}", serde_json::to_string(e)?)?; // one write call: atomic enough for O_APPEND
    Ok(())
}
