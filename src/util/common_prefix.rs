/// Return length of longest common prefix
pub fn len<S1, S2, T>(s1: S1, s2: S2) -> usize
where
    S1: Iterator<Item = T>,
    S2: Iterator<Item = T>,
    T: PartialEq,
{
    s1.zip(s2).take_while(|(c1, c2)| c1 == c2).count()
}

/// Return length of longest common prefix ending with an item satisfying the predicate
pub fn len_ending<S1, S2, T, F>(s1: S1, s2: S2, f: F) -> usize
where
    S1: Iterator<Item = T>,
    S2: Iterator<Item = T>,
    T: PartialEq,
    F: Fn(&T) -> bool,
{
    s1.zip(s2)
        .enumerate()
        .take_while(|(_i, (c1, c2))| c1 == c2)
        .filter(|(_, (c, _))| f(c))
        .last()
        .map(|(i, _)| i + 1)
        .unwrap_or(0)
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
        assert_eq!(len(s1.chars(), s2.chars()), expected);
    }

    #[test_case("a", "b", 0)]
    #[test_case("a", "a", 0)]
    #[test_case("a/", "a/", 2)]
    #[test_case("a/b/c/d", "a/b/c/d/e", 6)]
    fn test_len_ending(s1: &str, s2: &str, expected: usize) {
        // let f = |c| c == &'/';
        assert_eq!(len_ending(s1.chars(), s2.chars(), |c| c == &'/'), expected);
    }
}
