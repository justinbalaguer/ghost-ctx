/// Fast BPE-ish heuristic: ~4 chars per token.
pub fn estimate(s: &str) -> usize {
    s.len().div_ceil(4)
}

#[cfg(test)]
mod tests {
    #[test]
    fn rounds_up() {
        assert_eq!(super::estimate(""), 0);
        assert_eq!(super::estimate("abcde"), 2);
    }
}
