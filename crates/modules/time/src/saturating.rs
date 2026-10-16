//! One way to turn a size into the `u32` counts the API reports.

/// `n` as a `u32`, or `u32::MAX` when it does not fit (a count never panics or wraps).
pub fn saturating_u32<T: TryInto<u32>>(n: T) -> u32 {
    n.try_into().unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_size_that_fits_is_unchanged() {
        assert_eq!(saturating_u32(7_usize), 7);
        assert_eq!(saturating_u32(7_u64), 7);
    }

    #[test]
    fn a_size_that_does_not_fit_saturates() {
        assert_eq!(saturating_u32(u64::MAX), u32::MAX);
        assert_eq!(saturating_u32(-1_i64), u32::MAX);
    }
}
