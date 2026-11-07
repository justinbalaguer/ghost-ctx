//! Claude Code hook adapters (JSON on stdin, JSON / exit code on stdout / stderr).
mod condense;
mod guard;
mod payload;
mod post;
mod pre;
#[cfg(test)]
mod tests;

use payload::Input;
use std::io::Read;

fn read_input() -> Option<Input> {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).ok()?;
    serde_json::from_str(&s).ok()
}

/// Hooks never break the agent: unparseable input means "do nothing".
pub fn pre() -> anyhow::Result<()> {
    let Some(inp) = read_input() else { return Ok(()) };
    match pre::decide(&inp, &crate::ledger::home()) {
        pre::Decision::Allow => {}
        pre::Decision::Block(reason) => {
            eprintln!("{reason}");
            std::process::exit(2);
        }
        pre::Decision::Json(v) => println!("{v}"),
    }
    Ok(())
}

pub fn post() -> anyhow::Result<()> {
    let Some(inp) = read_input() else { return Ok(()) };
    if let Some(v) = post::decide(&inp, &crate::ledger::home()) {
        println!("{v}");
    }
    Ok(())
}
