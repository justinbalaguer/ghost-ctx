use regex::Regex;
use std::ops::Range;
use std::sync::LazyLock;

pub struct Finding {
    pub range: Range<usize>,
    pub kind: &'static str,
}

const PATTERNS: &[(&str, &str)] = &[
    ("private-key", r"(?s)-----BEGIN (?:[A-Z]+ )*PRIVATE KEY-----.*?-----END (?:[A-Z]+ )*PRIVATE KEY-----"),
    ("aws-access-key", r"\b(?:AKIA|ASIA|AGPA|AIDA|AROA)[0-9A-Z]{16}\b"),
    ("github-token", r"\bgh[pousr]_[A-Za-z0-9]{36,}\b|\bgithub_pat_[A-Za-z0-9_]{22,}"),
    ("anthropic-key", r"sk-ant-[A-Za-z0-9_\-]{20,}"),
    ("slack-token", r"\bxox[abprs]-[A-Za-z0-9\-]{10,}"),
];

static RES: LazyLock<Vec<(&str, Regex)>> =
    LazyLock::new(|| PATTERNS.iter().map(|(k, p)| (*k, Regex::new(p).unwrap())).collect());
static TOKEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[A-Za-z0-9+/_\-]{20,}={0,2}").unwrap());

pub fn shannon(s: &str) -> f64 {
    let mut counts = [0usize; 256];
    for b in s.bytes() {
        counts[b as usize] += 1;
    }
    let n = s.len() as f64;
    counts.iter().filter(|&&c| c > 0).map(|&c| {
        let p = c as f64 / n;
        -p * p.log2()
    }).sum()
}

fn looks_secret(t: &str) -> bool {
    if t.starts_with("REDACTED_") {
        return false;
    }
    let digit = t.bytes().any(|b| b.is_ascii_digit());
    let alpha = t.bytes().any(|b| b.is_ascii_alphabetic());
    if !(digit && alpha) {
        return false;
    }
    let hex = t.bytes().all(|b| b.is_ascii_hexdigit());
    if hex {
        // shortcut: len 40 skipped so git SHAs survive; sha256 etc. still redacted.
        return t.len() >= 32 && t.len() != 40 && shannon(t) >= 3.0;
    }
    shannon(t) >= 4.3
}

/// Non-overlapping findings, sorted by start.
pub fn find(text: &str) -> Vec<Finding> {
    let mut all: Vec<Finding> = Vec::new();
    for (kind, re) in RES.iter() {
        all.extend(re.find_iter(text).map(|m| Finding { range: m.range(), kind }));
    }
    for m in TOKEN.find_iter(text) {
        if looks_secret(m.as_str()) {
            all.push(Finding { range: m.range(), kind: "high-entropy" });
        }
    }
    // Earlier start wins; on tie the longer match wins.
    all.sort_by(|a, b| a.range.start.cmp(&b.range.start).then(b.range.end.cmp(&a.range.end)));
    let mut out: Vec<Finding> = Vec::new();
    for f in all {
        if out.last().is_none_or(|l| f.range.start >= l.range.end) {
            out.push(f);
        }
    }
    out
}
