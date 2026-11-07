use super::detect::find;
use crate::{prune, tokens};
use std::path::Path;

const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules"];
const BIG_TOKENS: usize = 1500;
const BIG_LINES: usize = 200;

pub fn scan(path: &Path) -> anyhow::Result<()> {
    let (mut secrets, mut big) = (0, 0);
    visit(path, &mut |p| {
        let Ok(text) = std::fs::read_to_string(p) else { return }; // binary/unreadable: skip
        for f in find(&text) {
            let line = text[..f.range.start].matches('\n').count() + 1;
            println!("SECRET {}:{line} {}", p.display(), f.kind); // never print the value
            secrets += 1;
        }
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        let (toks, lines) = (tokens::estimate(&text), text.lines().count());
        if prune::is_supported(ext) && (toks > BIG_TOKENS || lines > BIG_LINES) {
            println!("LARGE  {} (~{toks} tokens, {lines} lines)", p.display());
            big += 1;
        }
    })?;
    println!("{secrets} secret(s), {big} large file(s)");
    Ok(())
}

fn visit(p: &Path, f: &mut dyn FnMut(&Path)) -> anyhow::Result<()> {
    if p.is_dir() {
        for e in std::fs::read_dir(p)? {
            let e = e?.path();
            let skip = e.file_name().and_then(|n| n.to_str()).is_some_and(|n| SKIP_DIRS.contains(&n));
            if !skip {
                visit(&e, f)?;
            }
        }
    } else {
        f(p);
    }
    Ok(())
}
