use super::lang::{Kind, Lang, for_ext};
use tree_sitter::{Node, Parser};

const LONG_STRING: usize = 80;
const LONG_ARRAY_LINES: usize = 10;

struct Edit {
    range: std::ops::Range<usize>,
    text: String,
}

fn lines(n: Node) -> usize {
    n.end_position().row - n.start_position().row + 1
}

/// Skeleton of `src`, or None for unsupported ext / parse failure.
pub fn skeleton(src: &str, ext: &str) -> Option<String> {
    let lang = for_ext(ext)?;
    let mut parser = Parser::new();
    parser.set_language(&lang.ts).ok()?;
    let tree = parser.parse(src, None)?;
    let mut edits = Vec::new();
    walk(tree.root_node(), src, &lang, &mut edits);
    edits.sort_by_key(|e| e.range.start);

    let mut out = String::with_capacity(src.len() / 2);
    let mut pos = 0;
    for e in edits {
        out.push_str(&src[pos..e.range.start]);
        out.push_str(&e.text);
        pos = e.range.end;
    }
    out.push_str(&src[pos..]);
    Some(tidy(&out))
}

fn walk(n: Node, src: &str, lang: &Lang, edits: &mut Vec<Edit>) {
    let kind = n.kind();
    if lang.comments.contains(&kind) {
        if !lang.keep_comment(n, src) {
            edits.push(Edit { range: n.byte_range(), text: String::new() });
        }
        return;
    }
    if lang.fns.contains(&kind) {
        if let Some(body) = n.child_by_field_name("body").filter(|b| lang.blocks.contains(&b.kind())) {
            if let Some(e) = collapse_body(body, lang) {
                edits.push(e);
            }
            return; // signature kept, body gone: nothing deeper to visit
        }
    } else if lang.strings.contains(&kind) {
        let n_chars = n.byte_range().len();
        let docstring = n.parent().is_some_and(|p| p.kind() == "expression_statement");
        if n_chars > LONG_STRING && !docstring {
            let q = if kind == "template_string" || kind == "raw_string_literal" && lang.kind == Kind::Go {
                '`'
            } else {
                '"'
            };
            edits.push(Edit { range: n.byte_range(), text: format!("{q}[{n_chars} chars]{q}") });
            return;
        }
    } else if lang.arrays.contains(&kind) && lines(n) > LONG_ARRAY_LINES {
        let stub = format!("/* [collapsed {} lines] */", lines(n));
        let text = match lang.kind {
            Kind::Python => "[...]".into(),
            Kind::Go => format!("{{ {stub} }}"),
            _ => format!("[ {stub} ]"),
        };
        edits.push(Edit { range: n.byte_range(), text });
        return;
    }
    let mut c = n.walk();
    for ch in n.children(&mut c) {
        walk(ch, src, lang, edits);
    }
}

fn collapse_body(body: Node, lang: &Lang) -> Option<Edit> {
    if lang.kind == Kind::Python {
        // Keep a leading docstring; replace the rest.
        let mut c = body.walk();
        let stmts: Vec<Node> = body.named_children(&mut c).collect();
        let is_doc = |s: &Node| {
            s.kind() == "expression_statement" && s.named_child(0).is_some_and(|x| x.kind() == "string")
        };
        let skip = usize::from(stmts.first().is_some_and(is_doc));
        let first = stmts.get(skip)?;
        let n = body.end_position().row - first.start_position().row + 1;
        if n < 2 && skip == 0 {
            return None; // one-liner: nothing to save
        }
        return Some(Edit {
            range: first.start_byte()..body.end_byte(),
            text: format!("...  # [collapsed {n} lines]"),
        });
    }
    if lines(body) < 3 {
        return None;
    }
    let n = lines(body) - 2;
    Some(Edit { range: body.byte_range(), text: format!("{{ /* [collapsed {n} lines] */ }}") })
}

/// Trim trailing space and squeeze blank runs left behind by stripped comments.
fn tidy(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut blank = 0;
    for l in s.lines() {
        let l = l.trim_end();
        if l.is_empty() {
            blank += 1;
            if blank > 1 || out.is_empty() {
                continue;
            }
        } else {
            blank = 0;
        }
        out.push_str(l);
        out.push('\n');
    }
    out
}
