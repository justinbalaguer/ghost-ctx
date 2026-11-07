use tree_sitter::{Language, Node};

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Ts,
    Rust,
    Python,
    Go,
}

pub struct Lang {
    pub kind: Kind,
    pub ts: Language,
    /// Nodes whose `body` field is collapsed.
    pub fns: &'static [&'static str],
    /// Node kinds that count as a collapsible body.
    pub blocks: &'static [&'static str],
    pub comments: &'static [&'static str],
    pub strings: &'static [&'static str],
    pub arrays: &'static [&'static str],
}

const TS_FNS: &[&str] = &[
    "function_declaration",
    "generator_function_declaration",
    "method_definition",
    "arrow_function",
    "function_expression",
    "generator_function",
];

pub fn for_ext(ext: &str) -> Option<Lang> {
    let ts = |ts: Language| Lang {
        kind: Kind::Ts,
        ts,
        fns: TS_FNS,
        blocks: &["statement_block"],
        comments: &["comment"],
        strings: &["string", "template_string"],
        arrays: &["array"],
    };
    Some(match ext {
        "ts" | "mts" | "cts" => ts(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "tsx" => ts(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "js" | "jsx" | "mjs" | "cjs" => ts(tree_sitter_javascript::LANGUAGE.into()),
        "rs" => Lang {
            kind: Kind::Rust,
            ts: tree_sitter_rust::LANGUAGE.into(),
            fns: &["function_item", "closure_expression"],
            blocks: &["block"],
            comments: &["line_comment", "block_comment"],
            strings: &["string_literal", "raw_string_literal"],
            arrays: &["array_expression"],
        },
        "py" => Lang {
            kind: Kind::Python,
            ts: tree_sitter_python::LANGUAGE.into(),
            fns: &["function_definition"],
            blocks: &["block"],
            comments: &["comment"],
            strings: &["string"],
            arrays: &["list"],
        },
        "go" => Lang {
            kind: Kind::Go,
            ts: tree_sitter_go::LANGUAGE.into(),
            fns: &["function_declaration", "method_declaration", "func_literal"],
            blocks: &["block"],
            comments: &["comment"],
            strings: &["interpreted_string_literal", "raw_string_literal"],
            arrays: &["literal_value"],
        },
        _ => return None,
    })
}

impl Lang {
    /// Documentation comments survive pruning.
    pub fn keep_comment(&self, n: Node, src: &str) -> bool {
        let t = &src[n.byte_range()];
        match self.kind {
            Kind::Ts => t.starts_with("/**"),
            Kind::Rust => {
                (t.starts_with("///") && !t.starts_with("////"))
                    || t.starts_with("//!")
                    || t.starts_with("/**")
                    || t.starts_with("/*!")
            }
            Kind::Python => n.start_position().row == 0 && (t.starts_with("#!") || t.contains("coding")),
            // Go docs are the comment block directly above a declaration.
            Kind::Go => {
                let e = n.end_position();
                let last = if e.column == 0 { e.row.saturating_sub(1) } else { e.row };
                n.next_named_sibling().is_some_and(|s| s.start_position().row == last + 1)
            }
        }
    }
}
