/// Returns whether the sequence `needle` is a part of `haystack`, regardless of
/// its position.
///
/// # Arguments
///
/// * `haystack` - The sequence to look for `needle`
/// * `needle` - The sequence that may be a part of `haystack`
///
/// # Examples
///
/// ```ignore
/// use telnet_server::iter::contains_sequence;
/// // [1, 2, 3] is included in [1, 2, 3, 4, 5] -> returns true
/// assert!(contains_sequence(&[1, 2, 3, 4, 5], &[1, 2, 3]));
/// // [3, 3, 3] is *NOT* included in [1, 2, 3, 4, 5] -> returns false
/// assert!(!contains_sequence(&[1, 2, 3, 4, 5], &[3, 3, 3]));
/// ```
pub fn contains_sequence<T: Eq>(haystack: &[T], needle: &[T]) -> bool {
    let haystack_len = haystack.len();
    let needle_len = needle.len();

    if needle_len > haystack_len {
        return false;
    }

    if haystack_len == 0 {
        /* this would be an incorrect edge case otherwise */
        return false;
    }

    let size_diff = haystack_len - needle_len;

    'outer: for haystack_start in 0..=size_diff {
        for needle_index in 0..needle_len {
            if haystack[needle_index + haystack_start] != needle[needle_index] {
                continue 'outer;
            }
        }

        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_match_start() {
        assert!(contains_sequence(&[1, 2, 3, 4, 5], &[1, 2, 3]));
    }

    #[test]
    fn finds_match_middle() {
        assert!(contains_sequence(&[1, 2, 3, 4, 5], &[2, 3, 4]));
    }

    #[test]
    fn finds_match_end() {
        assert!(contains_sequence(&[1, 2, 3, 4, 5], &[3, 4, 5]));
    }

    #[test]
    fn finds_no_match() {
        assert!(!contains_sequence(&[1, 2, 3, 4, 5], &[3, 3, 3]));
    }

    #[test]
    fn finds_match_empty() {
        assert!(contains_sequence(&[1, 2, 3, 4, 5], &[]));
    }

    #[test]
    fn finds_no_match_on_empty_haystack() {
        assert!(!contains_sequence(&[], &[1]));
    }

    #[test]
    fn finds_no_match_on_empty_haystack_and_needle() {
        assert!(!contains_sequence::<u8>(&[], &[]));
    }
}
