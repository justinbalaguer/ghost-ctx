use super::Entry;
use std::path::Path;

/// Assumed input price, USD per million tokens. Edit to match your model.
const USD_PER_MTOK: f64 = 3.0;

pub fn stats() -> anyhow::Result<()> {
    print!("{}", render(&load(&super::home())));
    Ok(())
}

fn load(dir: &Path) -> Vec<Entry> {
    std::fs::read_to_string(dir.join("ledger.jsonl"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok()) // skip corrupt lines
        .collect()
}

fn row(label: &str, es: &[&Entry]) -> String {
    let orig: usize = es.iter().map(|e| e.original_token_estimate).sum();
    let kept: usize = es.iter().map(|e| e.skeleton_token_estimate).sum();
    let saved: usize = es.iter().map(|e| e.tokens_saved).sum();
    let pct = if orig == 0 { 0.0 } else { saved as f64 * 100.0 / orig as f64 };
    let usd = saved as f64 * USD_PER_MTOK / 1e6;
    format!("| {label:<9} | {:>6} | {orig:>10} | {kept:>10} | {saved:>10} | {pct:>5.1}% | ${usd:>8.4} |\n", es.len())
}

fn render(all: &[Entry]) -> String {
    // "Session" = the session of the most recent entry.
    let last = all.iter().max_by_key(|e| e.timestamp).map(|e| e.session_id.as_str());
    let session: Vec<&Entry> = all.iter().filter(|e| Some(e.session_id.as_str()) == last).collect();
    let everything: Vec<&Entry> = all.iter().collect();
    let sep = "+-----------+--------+------------+------------+------------+--------+-----------+\n";
    format!(
        "{sep}| Scope     | Events |   Original |       Kept |      Saved |  Saved | USD saved |\n{sep}{}{}{sep}\
         Token counts are estimates (~4 chars/token); USD at ${USD_PER_MTOK:.2}/Mtok input.\n",
        row("Session", &session),
        row("All-time", &everything),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_session_and_all_time() {
        let d = tempfile::tempdir().unwrap();
        let mut a = Entry::new("old", "read", "a.ts", 10_000, 2_000);
        a.timestamp = 1;
        let mut b = Entry::new("new", "output", "cargo test", 5_000, 1_000);
        b.timestamp = 2;
        for e in [&a, &b] {
            super::super::append_to(d.path(), e).unwrap();
        }
        std::fs::write(d.path().join("ledger.jsonl"), format!("garbage\n{}\n{}\n",
            serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap())).unwrap();
        let out = render(&load(d.path()));
        let session = out.lines().find(|l| l.contains("Session")).unwrap();
        let all = out.lines().find(|l| l.contains("All-time")).unwrap();
        assert!(session.contains("4000") && session.contains("80.0%"), "{session}");
        assert!(all.contains("12000") && all.contains("$  0.0360"), "{all}");
    }

    #[test]
    fn empty_ledger_renders() {
        let d = tempfile::tempdir().unwrap();
        assert!(render(&load(d.path())).contains("All-time"));
    }
}
