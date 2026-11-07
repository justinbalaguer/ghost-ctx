use regex::Regex;
use std::sync::LazyLock;

static NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?:^|[\s/'"=])(\.env(?:\.[\w.-]+)?|id_(?:rsa|dsa|ecdsa|ed25519)[\w.-]*|credentials(?:\.json)?|\.netrc|\.pgpass)(?:$|[\s'";|&)])"#,
    )
    .unwrap()
});
static READER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(cat|less|more|head|tail|bat|nl|tac|xxd|od|strings|base64|grep|rg|awk|sed|cut|source|cp|scp)\b|^\s*\.\s")
        .unwrap()
});

/// Reason to reject `cmd`, if it would dump a sensitive file.
pub fn check(cmd: &str) -> Option<String> {
    let hit = NAME.captures_iter(cmd).map(|c| c[1].to_string()).find(|n| {
        let templ = n.starts_with(".env.") && ["example", "sample", "template"].iter().any(|s| n.ends_with(s));
        !templ && !n.ends_with(".pub")
    })?;
    READER.is_match(cmd).then(|| {
        format!("ghost-ctx: blocked command that reads sensitive file `{hit}`. Ask the user if access is really needed.")
    })
}
