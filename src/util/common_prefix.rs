/// Return length of longest common prefix
pub fn len<S1, S2>(s1: S1, s2: S2) -> usize
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    s1.as_ref()
        .chars()
        .zip(s2.as_ref().chars())
        .take_while(|(c1, c2)| c1 == c2)
        .count()
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case("a", "b", 0)]
    #[test_case("", "b", 0)]
    #[test_case("a", "", 0)]
    #[test_case("a", "a", 1)]
    #[test_case("a1", "a2", 1)]
    #[test_case("abc", "abcd", 3)]
    fn test_len(s1: &str, s2: &str, expected: usize) {
        assert_eq!(len(s1, s2), expected);
    }
}
