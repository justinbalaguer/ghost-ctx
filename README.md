# ghost-ctx

Local token saver for Claude Code. Written in Rust, no network, no telemetry.

When a Claude Code agent reads a big source file, `ghost-ctx` hands it a **structural skeleton** instead (signatures and types kept, function bodies collapsed, secrets redacted). If the agent needs the full code, it asks again and gets it.

It also trims noisy test output and blocks shell commands that would dump credential files.

> Status: early (v0.1.0). Built for Claude Code hooks. Token numbers are estimates (~4 chars per token).
> **Not fully tested yet.** The unit tests pass, but savings have only been measured on a small number of reads. Real-world numbers will vary. Please report what you see.

---

## Quick start

You need [Rust](https://rustup.rs) installed (it provides `cargo`).

```bash
# 1. install (run this inside a clone of this repo)
git clone <this-repo-url>
cd ghost-ctx
cargo install --path .

# 2. register the hooks in the project you want to optimize
cd your-project
ghost-ctx init          # writes/merges .claude/settings.json

# 3. work as usual, then check savings
ghost-ctx stats
```

What `cargo install --path .` means: `cargo install` builds the program and copies the `ghost-ctx` binary into `~/.cargo/bin`. `--path .` tells it to build the code in the current folder (the clone) instead of downloading a published crate. Make sure `~/.cargo/bin` is on your `PATH`, then check with `ghost-ctx --version`. To update later, `git pull` and run the same command again with `--force`.

`init` is idempotent and keeps your existing settings and hooks. If `settings.json` is not valid JSON it refuses to touch it.

---

## What it does

### 1. Skeleton reads (`PreToolUse` on `Read`)

A file is replaced by its skeleton when **all** of these hold:

| Condition | Value |
|---|---|
| Language supported | TypeScript/TSX, JavaScript/JSX, Rust, Python, Go |
| File is big | more than 1500 estimated tokens **or** more than 200 lines |
| Read is plain | no `offset`/`limit`, and `GHOST_CTX_FULL` is not set |
| Not already tried | first read of this file in this session |
| Worth it | skeleton is at least 10% smaller |

How the agent gets the skeleton: the hook **denies** the first `Read` and puts the skeleton in the denial's `additionalContext`, with a note explaining what happened.

How the agent gets the real code (any one works):

- Read the same file again. The second attempt in the same session is allowed through.
- Read with `offset`/`limit`. Ranged reads are never intercepted.
- Set `GHOST_CTX_FULL=1`.

What the skeleton keeps: signatures, types, imports, doc comments. What it drops: function bodies, ordinary comments, long strings, long arrays.

### 2. Secret redaction

Skeletons are scanned before the agent sees them. Matches become `REDACTED_KEY_01`, `REDACTED_KEY_02`, ... (same secret, same placeholder, within a call).

Detected: private keys, AWS access keys, GitHub tokens, Anthropic keys, Slack tokens, plus long high-entropy strings.

This is heuristic. It is a safety net, not a security boundary. Do not rely on it to protect real secrets.

### 3. Sensitive-file guard (`PreToolUse` on `Bash`)

Blocks (exit code 2) shell commands that read files such as `.env`, `.env.*`, `id_rsa`/`id_ed25519`/etc., `credentials`, `.netrc`, `.pgpass` with tools like `cat`, `grep`, `head`, `sed`, `cp`, `source`.
Allowed: `.env.example`/`.sample`/`.template` and `.pub` files.
This is regex based and can be bypassed. It catches accidents, not attackers.

### 4. Test-output condensing (`PostToolUse` on `Bash`)

For test commands (`cargo test`, `pytest`, `go test`, `npm test`, `jest`, `vitest`, ...) with output over 2000 characters, passing and progress lines are dropped. Failures stay. A header says how many lines were removed.

### 5. Ledger and stats

Every saving is appended to `~/.ghost-ctx/ledger.jsonl`. `ghost-ctx stats` prints a session and all-time table.

```
| Scope     | Events |   Original |       Kept |      Saved |  Saved | USD saved |
| Session   |      1 |       3105 |       1782 |       1323 |  42.6% | $  0.0040 |
```

USD uses an assumed $3.00 per million input tokens (constant in `src/ledger/stats.rs`).

---

## Commands

| Command | What it does |
|---|---|
| `ghost-ctx init` | Register hooks in `./.claude/settings.json` |
| `ghost-ctx stats` | Show token and cost savings |
| `ghost-ctx prune <file>` | Print the skeleton of a file (try it without any agent) |
| `ghost-ctx scan <path>` | List suspected secrets (never prints values) and large files |
| `ghost-ctx hook pre` / `hook post` | Hook handlers. Called by Claude Code, read JSON on stdin |

Environment variables:

| Variable | Effect |
|---|---|
| `GHOST_CTX_FULL` | If set, never replace reads with skeletons |
| `GHOST_CTX_HOME` | Override `~/.ghost-ctx` (ledger and state) |

---

## When it helps, and when it doesn't

- **Helps:** exploring. "What is in this file?", "what functions exist?" on large files. The agent pays for the outline, not the whole file.
- **Little effect:** focused fixes where the agent already greps and reads a few lines with `offset`/`limit`. Nothing big is read, so nothing is shrunk.
- **Can cost extra:** the agent reads a whole file, gets the skeleton, then must read again for the real code. That is one more step and more tokens than a single full read.

The skeleton only shows what exists, not what it does. Code that is changed should be read in full first.

Measure it on your own work with `ghost-ctx stats` before trusting any number.

---

## For AI agents

If you see `ghost-ctx: an optimized structural skeleton of this file was provided`:

1. Bodies are collapsed (`...  # [collapsed N lines]`) and secrets are replaced. The file on disk is unchanged.
2. Names and signatures are reliable. Behavior is not visible. Do not guess what a collapsed body does.
3. To answer questions about structure, use the skeleton as is.
4. Before editing or explaining logic, read the real code: read the file again, or use `offset`/`limit` for just the lines you need.
5. Redacted values (`REDACTED_KEY_NN`) are placeholders. Never write them into files.

If a Bash command is blocked with `ghost-ctx: blocked command that reads sensitive file`, ask the user instead of working around it.

Hooks never crash the agent: input they cannot parse is ignored and the tool call proceeds normally.

---

## Project layout

```
src/
  main.rs      CLI entry (clap)
  prune/       tree-sitter skeleton engine, per-language rules
  secrets/     detection, redaction, `scan`
  hooks/       Claude Code hook adapters: pre (read/bash), post (test output)
  init/        settings.json registration
  ledger/      JSONL ledger and `stats`
  tokens.rs    ~4 chars/token estimate
```

Each module has its own tests.

```bash
cargo test    # 20 tests
```

---

## Limitations

- Claude Code only (hook JSON format). Other agents are not supported yet.
- Five language families. Other files are read normally.
- Token counts are a rough estimate, not a real tokenizer.
- Secret detection and the Bash guard are heuristics.
- Whether subagents' reads pass through the hook has not been verified.

---

## Contributing

Issues and pull requests are welcome. Run `cargo test` and keep new features in their own module with tests.

## License

[MIT](LICENSE)
