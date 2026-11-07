use super::{condense, payload::Input};
use crate::{ledger, tokens};
use serde_json::{Value, json};
use std::path::Path;

pub fn decide(inp: &Input, home: &Path) -> Option<Value> {
    if inp.tool_name != "Bash" {
        return None;
    }
    let cmd = inp.str_field("command")?;
    if !condense::is_test_command(cmd) {
        return None;
    }
    let mut resp = inp.tool_response.clone();
    let raw = resp.get("stdout")?.as_str()?;
    let out = condense::condense(raw)?;
    let label: String = cmd.chars().take(80).collect();
    let entry = ledger::Entry::new(&inp.session_id, "output", &label, tokens::estimate(raw), tokens::estimate(&out));
    let _ = ledger::append_to(home, &entry);
    resp["stdout"] = Value::String(out);
    Some(json!({"hookSpecificOutput": {"hookEventName": "PostToolUse", "updatedToolOutput": resp}}))
}
