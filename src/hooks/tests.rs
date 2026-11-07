use super::{payload::Input, post, pre};
use serde_json::json;

fn inp(v: serde_json::Value) -> Input {
    serde_json::from_value(v).unwrap()
}

fn big_ts(dir: &std::path::Path) -> String {
    let body: String = (0..300).map(|i| format!("  const v{i} = compute({i});\n")).collect();
    let p = dir.join("big.ts");
    std::fs::write(&p, format!("export function f() {{\n{body}}}\n")).unwrap();
    p.to_str().unwrap().into()
}

#[test]
fn read_denies_with_skeleton_then_allows_second_attempt() {
    let d = tempfile::tempdir().unwrap();
    let path = big_ts(d.path());
    let i = inp(json!({"session_id":"s","hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":path}}));
    let pre::Decision::Json(v) = pre::decide(&i, d.path()) else { panic!("want deny") };
    let h = &v["hookSpecificOutput"];
    assert_eq!(h["hookEventName"], "PreToolUse");
    assert_eq!(h["permissionDecision"], "deny");
    assert!(h["additionalContext"].as_str().unwrap().contains("[collapsed 300 lines]"));
    assert!(matches!(pre::decide(&i, d.path()), pre::Decision::Allow), "second attempt allowed");
    assert!(std::fs::read_to_string(d.path().join("ledger.jsonl")).unwrap().contains("\"tokens_saved\""));
}

#[test]
fn read_with_range_small_or_unknown_is_allowed() {
    let d = tempfile::tempdir().unwrap();
    let path = big_ts(d.path());
    let ranged = inp(json!({"tool_name":"Read","tool_input":{"file_path":path,"limit":50}}));
    assert!(matches!(pre::decide(&ranged, d.path()), pre::Decision::Allow));
    let small = d.path().join("s.ts");
    std::fs::write(&small, "export const a = 1;\n").unwrap();
    let i = inp(json!({"tool_name":"Read","tool_input":{"file_path":small}}));
    assert!(matches!(pre::decide(&i, d.path()), pre::Decision::Allow));
    let i = inp(json!({"tool_name":"Read","tool_input":{"file_path":"/x/y.md"}}));
    assert!(matches!(pre::decide(&i, d.path()), pre::Decision::Allow));
}

#[test]
fn bash_blocks_sensitive_dumps_only() {
    let d = tempfile::tempdir().unwrap();
    let run = |c: &str| pre::decide(&inp(json!({"tool_name":"Bash","tool_input":{"command":c}})), d.path());
    for c in ["cat .env", "head -n3 ~/.ssh/id_rsa", "grep KEY config/.env.production", "cat creds/credentials.json"] {
        assert!(matches!(run(c), pre::Decision::Block(_)), "should block: {c}");
    }
    for c in ["cat .env.example", "ls -la", "cat ~/.ssh/id_rsa.pub", "cargo build"] {
        assert!(matches!(run(c), pre::Decision::Allow), "should allow: {c}");
    }
}

#[test]
fn post_condenses_test_output() {
    let d = tempfile::tempdir().unwrap();
    let mut out: String = (0..200).map(|i| format!("test mod::t{i} ... ok\n")).collect();
    out.push_str("test mod::bad ... FAILED\n\nfailures:\n  mod::bad panicked at src/x.rs:1:1\n\ntest result: FAILED. 200 passed; 1 failed\n");
    let i = inp(json!({"session_id":"s","tool_name":"Bash","tool_input":{"command":"cargo test"},
        "tool_response":{"stdout":out,"stderr":"","interrupted":false,"isImage":false}}));
    let v = post::decide(&i, d.path()).expect("condensed");
    let new = &v["hookSpecificOutput"]["updatedToolOutput"];
    let s = new["stdout"].as_str().unwrap();
    assert!(s.contains("mod::bad ... FAILED") && s.contains("panicked at src/x.rs") && s.contains("1 failed"));
    assert!(!s.contains("t100 ... ok"));
    assert_eq!(new["interrupted"], false);
    let non_test = inp(json!({"tool_name":"Bash","tool_input":{"command":"ls"},"tool_response":{"stdout":out}}));
    assert!(post::decide(&non_test, d.path()).is_none());
}
