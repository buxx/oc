pub fn remove_numeric_suffix(s: &str) -> &str {
    // Find last '_' followed by only digits until end of string
    if let Some(pos) = s.rfind('_') {
        if s[pos + 1..].chars().all(|c| c.is_ascii_digit()) && !s[pos + 1..].is_empty() {
            return &s[..pos];
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_numeric_suffix() {
        assert_eq!(remove_numeric_suffix("toto_1"), "toto");
        assert_eq!(remove_numeric_suffix("toto_a_2"), "toto_a");
        assert_eq!(remove_numeric_suffix("foo_bar_42"), "foo_bar");
        assert_eq!(remove_numeric_suffix("no_suffix"), "no_suffix");
        assert_eq!(remove_numeric_suffix("trailing_"), "trailing_");
        assert_eq!(remove_numeric_suffix("abc_10_def"), "abc_10_def");
    }
}
