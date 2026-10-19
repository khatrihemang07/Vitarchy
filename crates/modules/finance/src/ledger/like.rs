//! The pattern for a case-insensitive "contains" search with `ILIKE`.

/// `%text%` with the characters LIKE treats specially (`%`, `_`, `\`) escaped, so the person's
/// text is matched literally. Postgres uses `\` as the default escape.
#[must_use]
pub fn contains_pattern(text: &str) -> String {
    let mut pattern = String::with_capacity(text.len() + 2);
    pattern.push('%');
    for c in text.chars() {
        if matches!(c, '%' | '_' | '\\') {
            pattern.push('\\');
        }
        pattern.push(c);
    }
    pattern.push('%');
    pattern
}

#[cfg(test)]
mod tests {
    use super::contains_pattern;

    #[test]
    fn plain_text_is_wrapped_in_wildcards() {
        assert_eq!(contains_pattern("rent"), "%rent%");
    }

    #[test]
    fn like_wildcards_and_the_escape_are_escaped() {
        assert_eq!(contains_pattern("50%_off\\"), "%50\\%\\_off\\\\%");
    }
}
