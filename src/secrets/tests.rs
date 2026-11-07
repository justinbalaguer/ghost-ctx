use super::Redactor;

fn red(s: &str) -> String {
    Redactor::default().redact(s)
}

#[test]
fn provider_patterns() {
    let aws = format!("AKIA{}", "IOSFODNN7EXAMPLE");
    let gh = format!("ghp_{}", "a1B2".repeat(9));
    let gh_pat = format!("github_pat_{}", "A1b2C3d4E5".repeat(3));
    let ant = format!("sk-ant-api03-{}", "abcDEF123_-".repeat(4));
    let slack = format!("xoxb-{}-{}", "123456789012", "abcdefghijklmnop");
    let pk = "-----BEGIN RSA PRIVATE KEY-----\nMIIEow\nabc\n-----END RSA PRIVATE KEY-----";
    for s in [aws, gh, gh_pat, ant, slack, pk.to_string()] {
        let out = red(&format!("key = {s}\n"));
        assert_eq!(out, "key = REDACTED_KEY_01\n", "not masked: {s}");
    }
}

#[test]
fn entropy_base64_and_hex() {
    assert_eq!(red("t=Zk9xQ2vL8mP4nR7tY1wB5cD3fG6hJ0sA"), "t=REDACTED_KEY_01");
    let sha256 = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
    assert_eq!(red(sha256), "REDACTED_KEY_01");
}

#[test]
fn leaves_normal_code_alone() {
    let code = "let very_long_identifier_name_here = compute_something_useful();\n\
                // commit 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b\n\
                path = src/some/long/path/to/file.rs\n\
                id = 123e4567-e89b-12d3-a456-426614174000\n";
    assert_eq!(red(code), code);
}

#[test]
fn deterministic_session_map() {
    let a = format!("AKIA{}", "IOSFODNN7EXAMPLE");
    let b = format!("ghp_{}", "a1B2".repeat(9));
    let mut r = Redactor::default();
    let out = r.redact(&format!("{a} {b} {a}"));
    assert_eq!(out, "REDACTED_KEY_01 REDACTED_KEY_02 REDACTED_KEY_01");
    // later call keeps earlier mapping
    assert_eq!(r.redact(&b), "REDACTED_KEY_02");
}
