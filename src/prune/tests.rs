use super::skeleton;
use crate::tokens::estimate;

fn body(n: usize, ind: &str, stmt: &str) -> String {
    (0..n).map(|i| format!("{ind}{}\n", stmt.replace("{i}", &i.to_string()))).collect()
}

#[test]
fn typescript() {
    let src = format!(
        "import {{ a }} from './a';\n// plain comment\n/** doc */\nexport interface Foo {{ x: number }}\nexport type Id = string;\nenum E {{ A, B }}\nexport class C {{\n  m(x: number): number {{\n{}  }}\n}}\nexport function f(a: string) {{\n{}}}\nconst g = () => {{\n{}}};\n",
        body(15, "    ", "console.log({i});"),
        body(15, "  ", "const v{i} = compute({i});"),
        body(15, "  ", "doThing({i});"),
    );
    let out = skeleton(&src, "ts").unwrap();
    assert!(out.contains("import { a }"));
    assert!(out.contains("export interface Foo { x: number }"));
    assert!(out.contains("export type Id = string;"));
    assert!(out.contains("enum E"));
    assert!(out.contains("/** doc */"));
    assert!(!out.contains("plain comment"));
    assert!(out.contains("m(x: number): number { /* [collapsed 15 lines] */ }"));
    assert!(out.contains("export function f(a: string) { /* [collapsed 15 lines] */ }"));
    assert!(out.contains("const g = () => { /* [collapsed 15 lines] */ };"));
    assert!(!out.contains("compute("));
    assert!(estimate(&out) * 100 < estimate(&src) * 40, "want >60% reduction");
}

#[test]
fn python() {
    let src = format!(
        "import os\n# strip me\nclass A:\n    \"\"\"Doc.\"\"\"\n    def m(self, x):\n        \"\"\"Method doc.\"\"\"\n{}\ndef f(a):\n{}\nDATA = [\n{}]\n",
        body(15, "        ", "print({i})"),
        body(15, "    ", "y{i} = {i}"),
        body(15, "    ", "{i},"),
    );
    let out = skeleton(&src, "py").unwrap();
    assert!(out.contains("import os"));
    assert!(out.contains("class A:"));
    assert!(out.contains("\"\"\"Doc.\"\"\""));
    assert!(out.contains("\"\"\"Method doc.\"\"\""));
    assert!(out.contains("def f(a):\n    ...  # [collapsed 15 lines]"));
    assert!(out.contains("DATA = [...]"));
    assert!(!out.contains("strip me") && !out.contains("print("));
    assert!(estimate(&out) * 100 < estimate(&src) * 40);
}

#[test]
fn rust() {
    let src = format!(
        "use std::fmt;\n// strip\n/// keep\npub struct S {{ a: u32 }}\npub enum E {{ A }}\ntype T = u8;\nimpl S {{\n    /// doc\n    pub fn m(&self) -> u32 {{\n{}    }}\n}}\nfn lit() -> &'static str {{\n    \"{}\"\n}}\n",
        body(15, "        ", "let v{i} = {i} * 2;"),
        "x".repeat(100),
    );
    let out = skeleton(&src, "rs").unwrap();
    assert!(out.contains("use std::fmt;"));
    assert!(out.contains("/// keep"));
    assert!(out.contains("pub struct S { a: u32 }"));
    assert!(out.contains("pub fn m(&self) -> u32 { /* [collapsed 15 lines] */ }"));
    assert!(!out.contains("// strip"));
    assert!(!out.contains("let v0"));
}

#[test]
fn long_string_and_array_outside_fn() {
    let src = format!("const S = \"{}\";\nconst A = [\n{}];\n", "y".repeat(120), body(12, "  ", "{i},"));
    let out = skeleton(&src, "js").unwrap();
    assert!(out.contains("\"[122 chars]\""));
    assert!(out.contains("[ /* [collapsed 14 lines] */ ]"));
}

#[test]
fn go() {
    let src = format!(
        "package main\n\n// stray\n\n// F does x.\nfunc F(a int) int {{\n{}}}\n\ntype T struct {{ A int }}\n",
        body(15, "\t", "_ = {i}"),
    );
    let out = skeleton(&src, "go").unwrap();
    assert!(out.contains("// F does x."));
    assert!(!out.contains("stray"));
    assert!(out.contains("func F(a int) int { /* [collapsed 15 lines] */ }"));
    assert!(out.contains("type T struct { A int }"));
}

#[test]
fn unsupported_ext() {
    assert!(skeleton("x", "txt").is_none());
}
