//! Reading a printed Account number: the trailing digits a mask shows, and whether two
//! masks show the same number.

/// Fewest visible trailing digits that are enough to compare two masks.
const MIN_VISIBLE_DIGITS: usize = 2;

/// The digits a mask ends in, as printed: `XXXXXX4321` gives `4321`, `XXXX XX56` gives `34`.
#[must_use]
pub fn visible_digits(mask: &str) -> String {
    let digits: Vec<char> = mask
        .trim()
        .chars()
        .rev()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.into_iter().rev().collect()
}

/// The last 4 digits of a printed mask, when the mask ends in at least 4 digits:
/// `XXXXXX4321`, `4123XXXXXXXX2003` and `XX9971` work; `1234XXXX` or no mask do not.
#[must_use]
pub fn last4(mask: Option<&str>) -> Option<String> {
    let digits = visible_digits(mask?);
    (digits.len() >= 4).then(|| digits[digits.len() - 4..].to_owned())
}

/// The trailing digits worth remembering for a mask: at least 2 are visible.
#[must_use]
pub fn remembered_digits(mask: Option<&str>) -> Option<String> {
    let digits = visible_digits(mask?);
    (digits.len() >= MIN_VISIBLE_DIGITS).then_some(digits)
}

/// Whether two masks show the same number. With 2 or more visible trailing digits on both,
/// the shorter run must be the end of the longer; otherwise the masks must be equal ignoring
/// case and spaces.
#[must_use]
pub fn same_number(a: &str, b: &str) -> bool {
    let (da, db) = (visible_digits(a), visible_digits(b));
    if da.len() >= MIN_VISIBLE_DIGITS && db.len() >= MIN_VISIBLE_DIGITS {
        let shared = da.len().min(db.len());
        return da[da.len() - shared..] == db[db.len() - shared..];
    }
    let (na, nb) = (squashed(a), squashed(b));
    !na.is_empty() && na == nb
}

fn squashed(mask: &str) -> String {
    mask.split_whitespace().collect::<String>().to_uppercase()
}
