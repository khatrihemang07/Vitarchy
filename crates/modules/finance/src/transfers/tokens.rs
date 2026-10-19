//! Reading descriptions: the words and digit runs in them, and whether they hold a marker.

/// The upper-cased runs of letters and digits in `text`.
pub fn words(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_uppercase)
}

/// The runs of ASCII digits in `text`.
pub fn digit_runs(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_ascii_digit())
        .filter(|run| !run.is_empty())
}

/// `text` as lower-case words joined and surrounded by single spaces, so a phrase is found
/// only as whole words and punctuation never matters.
#[must_use]
pub fn spaced(text: &str) -> String {
    let words: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect();
    format!(" {} ", words.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_split_on_punctuation_and_fold_case() {
        let found: Vec<String> = words("NEFT-ab12/cd, x").collect();
        assert_eq!(found, ["NEFT", "AB12", "CD", "X"]);
    }

    #[test]
    fn digit_runs_are_maximal() {
        let found: Vec<&str> = digit_runs("DR 0012345 AB 99-7").collect();
        assert_eq!(found, ["0012345", "99", "7"]);
    }

    #[test]
    fn spaced_text_finds_whole_words_only() {
        assert!(spaced("UPI/cred.club/pay").contains(&spaced("CRED Club")));
        assert!(!spaced("SECRED CLUBHOUSE").contains(&spaced("CRED Club")));
    }
}
