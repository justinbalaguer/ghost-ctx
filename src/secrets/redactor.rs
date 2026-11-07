use super::detect::find;
use std::collections::HashMap;

/// Session map: same secret always gets the same placeholder.
#[derive(Default)]
pub struct Redactor {
    map: HashMap<String, String>,
}

impl Redactor {
    pub fn redact(&mut self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut pos = 0;
        for f in find(text) {
            out.push_str(&text[pos..f.range.start]);
            let secret = &text[f.range.clone()];
            let next = self.map.len() + 1;
            let ph = self.map.entry(secret.to_string()).or_insert_with(|| format!("REDACTED_KEY_{next:02}"));
            out.push_str(ph);
            pos = f.range.end;
        }
        out.push_str(&text[pos..]);
        out
    }
}
