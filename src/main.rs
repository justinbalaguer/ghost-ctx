mod hooks;
mod init;
mod ledger;
mod prune;
mod secrets;
mod tokens;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ghost-ctx", version, about = "Local context/token optimizer for Claude Code")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Emit AST skeleton of a file to stdout
    Prune { file_path: PathBuf },
    /// Scan file or dir for credentials and large uncompressed files
    Scan { path: PathBuf },
    /// Claude Code hook handlers (stdin JSON)
    Hook {
        #[command(subcommand)]
        kind: HookKind,
    },
    /// Show token and cost savings
    Stats,
    /// Register hooks in .claude/settings.json
    Init,
}

#[derive(Subcommand)]
enum HookKind {
    Pre,
    Post,
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().cmd {
        Cmd::Prune { file_path } => prune::run(&file_path),
        Cmd::Scan { path } => secrets::scan(&path),
        Cmd::Hook { kind: HookKind::Pre } => hooks::pre(),
        Cmd::Hook { kind: HookKind::Post } => hooks::post(),
        Cmd::Stats => ledger::stats(),
        Cmd::Init => init::run(),
    }
}
