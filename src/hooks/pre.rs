use super::{guard, payload::Input};
use crate::{ledger, prune, secrets::Redactor, tokens};
use serde_json::{Value, json};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

const MAX_TOKENS: usize = 1500;
const MAX_LINES: usize = 200;

pub enum Decision {
    Allow,
    /// exit 2 with this stderr text
    Block(String),
    Json(Value),
}

pub fn decide(inp: &Input, home: &Path) -> Decision {
    match inp.tool_name.as_str() {
        "Bash" => match inp.str_field("command").and_then(guard::check) {
            Some(r) => Decision::Block(r),
            None => Decision::Allow,
        },
        "Read" => read(inp, home).map_or(Decision::Allow, Decision::Json),
        _ => Decision::Allow,
    }
}

fn read(inp: &Input, home: &Path) -> Option<Value> {
    let path = inp.str_field("file_path")?;
    // Ranged reads and GHOST_CTX_FULL are the explicit "give me the real thing" overrides.
    if inp.tool_input.get("offset").is_some()
        || inp.tool_input.get("limit").is_some()
        || std::env::var_os("GHOST_CTX_FULL").is_some()
    {
        return None;
    }
    let ext = Path::new(path).extension()?.to_str()?;
    if !prune::is_supported(ext) {
        return None;
    }
    let src = std::fs::read_to_string(path).ok()?;
    let orig = tokens::estimate(&src);
    if orig <= MAX_TOKENS && src.lines().count() <= MAX_LINES {
        return None;
    }
    // Second attempt on the same file in the same session: allow the full read.
    let mut h = DefaultHasher::new();
    (&inp.session_id, path).hash(&mut h);
    let marker = home.join("seen").join(format!("{:x}", h.finish()));
    if marker.exists() {
        let _ = std::fs::remove_file(&marker);
        return None;
    }
    let skel = Redactor::default().redact(&prune::skeleton(&src, ext)?);
    let kept = tokens::estimate(&skel);
    if kept * 10 >= orig * 9 {
        return None; // no real gain
    }
    let _ = std::fs::create_dir_all(marker.parent()?).and_then(|_| std::fs::write(&marker, ""));
    let _ = ledger::append_to(home, &ledger::Entry::new(&inp.session_id, "read", path, orig, kept));
    let note = "ghost-ctx: an optimized structural skeleton of this file was provided instead of the full source \
                (bodies collapsed, secrets redacted). Read the file again, or use offset/limit, for the full implementation.";
    Some(json!({"hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "deny",
        "permissionDecisionReason": note,
        "additionalContext": format!("{note}\n\n{skel}"),
    }}))
}
