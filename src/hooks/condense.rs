use regex::Regex;
use std::sync::LazyLock;

const MIN_CHARS: usize = 2000;

static RUNNER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(npm (run )?test|yarn test|pnpm (run )?test|npx (vitest|jest)|vitest|jest|cargo (test|nextest)|pytest|go test)\b")
        .unwrap()
});
static NOISE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\s*(test .+ \.\.\. ok|.*\bPASSED\b.*|\s*[✓✔√] .*|PASS .*|ok\s+\S+\s+[\d.]+s.*|\s*(Compiling|Downloaded|Downloading|Running|Finished|Fresh) .*|[.sFEx]+(\s*\[\s*\d+%\])?|.*\b\d{1,3}%\s*$)\s*$",
    )
    .unwrap()
});

pub fn is_test_command(cmd: &str) -> bool {
    RUNNER.is_match(cmd)
}

/// Drop passing/progress lines and consecutive duplicates; None when not worth it.
pub fn condense(raw: &str) -> Option<String> {
    if raw.len() < MIN_CHARS {
        return None;
    }
    let (mut dropped, mut prev) = (0usize, None::<&str>);
    let mut kept = Vec::new();
    for l in raw.lines() {
        if NOISE.is_match(l) || (prev == Some(l) && !l.trim().is_empty()) {
            dropped += 1;
        } else {
            kept.push(l);
        }
        prev = Some(l);
    }
    let out = format!("[ghost-ctx: dropped {dropped} passing/progress lines]\n{}", kept.join("\n"));
    (out.len() * 10 < raw.len() * 8).then_some(out)
}
