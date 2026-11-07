//! Registers ghost-ctx hooks in .claude/settings.json (idempotent, preserves other settings).
use anyhow::Context;
use serde_json::{Value, json};
use std::path::Path;

const HOOKS: &[(&str, &str, &str)] = &[
    ("PreToolUse", "Read|Bash", "ghost-ctx hook pre"),
    ("PostToolUse", "Bash", "ghost-ctx hook post"),
];

pub fn run() -> anyhow::Result<()> {
    let path = init_at(Path::new("."))?;
    println!("ghost-ctx hooks registered in {}", path.display());
    Ok(())
}

fn init_at(dir: &Path) -> anyhow::Result<std::path::PathBuf> {
    let path = dir.join(".claude").join("settings.json");
    let mut settings: Value = match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).with_context(|| format!("{} is not valid JSON; not touching it", path.display()))?,
        Err(_) => json!({}),
    };
    merge(&mut settings)?;
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(&path, serde_json::to_string_pretty(&settings)? + "\n")?;
    Ok(path)
}

fn merge(settings: &mut Value) -> anyhow::Result<()> {
    let hooks = settings
        .as_object_mut()
        .context("settings.json root is not an object")?
        .entry("hooks")
        .or_insert_with(|| json!({}));
    for (event, matcher, cmd) in HOOKS {
        let list = hooks
            .as_object_mut()
            .context("`hooks` is not an object")?
            .entry(*event)
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .with_context(|| format!("`hooks.{event}` is not an array"))?;
        let present = list.iter().any(|e| {
            e["hooks"].as_array().is_some_and(|hs| hs.iter().any(|h| h["command"] == *cmd))
        });
        if !present {
            list.push(json!({"matcher": matcher, "hooks": [{"type": "command", "command": cmd}]}));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(p: &Path) -> Value {
        serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
    }

    #[test]
    fn creates_merges_and_is_idempotent() {
        let d = tempfile::tempdir().unwrap();
        let p = init_at(d.path()).unwrap();
        let v = read(&p);
        assert_eq!(v["hooks"]["PreToolUse"][0]["hooks"][0]["command"], "ghost-ctx hook pre");
        assert_eq!(v["hooks"]["PostToolUse"][0]["matcher"], "Bash");
        init_at(d.path()).unwrap();
        assert_eq!(read(&p)["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn preserves_existing_settings_and_hooks() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join(".claude/settings.json");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, r#"{"model":"x","hooks":{"PreToolUse":[{"matcher":"Edit","hooks":[{"type":"command","command":"other"}]}]}}"#).unwrap();
        init_at(d.path()).unwrap();
        let v = read(&p);
        assert_eq!(v["model"], "x");
        assert_eq!(v["hooks"]["PreToolUse"].as_array().unwrap().len(), 2);
        assert_eq!(v["hooks"]["PreToolUse"][0]["hooks"][0]["command"], "other");
    }

    #[test]
    fn invalid_json_is_left_alone() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join(".claude/settings.json");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "{ nope").unwrap();
        assert!(init_at(d.path()).is_err());
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "{ nope");
    }
}
