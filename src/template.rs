use std::{collections::HashMap, fmt::Display};

#[derive(PartialEq, Debug)]
pub enum Error {
    UnknownFormatCharacter(char),
    TrailingPercent,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            UnknownFormatCharacter(c) => write!(f, "Unknown format character {}", c),
            TrailingPercent => f.write_str("Malformed percent sequence at end of string"),
        }
    }
}

impl std::error::Error for Error {}

/// Simple percent-oriented templating, where percent followed by any character is
/// subtituted by the corresponding string from values.
pub fn format<S1, S2>(format_str: S1, values: HashMap<char, S2>) -> Result<String, Error>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let format_str = format_str.as_ref();
    let mut percent = false;
    let mut formatted = String::new();

    for c in format_str.chars() {
        if c == '%' {
            if percent {
                formatted.push(c);
                percent = false;
            } else {
                percent = true;
            }
        } else if percent {
            match values.get(&c) {
                Some(value) => {
                    formatted.push_str(value.as_ref());
                }
                None => return Err(Error::UnknownFormatCharacter(c)),
            }
            percent = false;
        } else {
            formatted.push(c);
        }
    }

    if percent {
        Err(Error::TrailingPercent)
    } else {
        Ok(formatted)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use test_case::test_case;

    use super::{Error::*, *};

    #[test_case("Hello Template World!", [], Ok("Hello Template World!"))]
    #[test_case("Hello Template World!%", [], Err(TrailingPercent))]
    #[test_case("Hello Template World!%%", [], Ok("Hello Template World!%"); "trailing percent percent")]
    #[test_case("Hello %s World!", [], Err(UnknownFormatCharacter('s')))]
    #[test_case("Hello %s World!", [('s', "Template")], Ok("Hello Template World!"))]
    #[test_case("Hello %s%% World!", [('s', "Template")], Ok("Hello Template% World!"); "percent percent")]
    #[test_case("Hello %a%b%a%c World!", [('a', "A"), ('b', "B"), ('c', "C")], Ok("Hello ABAC World!"))]
    fn test_format<'i, I>(format_str: &str, values: I, expected: Result<&str, Error>)
    where
        I: IntoIterator<Item = (char, &'i str)>,
    {
        let values = HashMap::from_iter(values);
        assert_eq!(format(format_str, values), expected.map(|s| s.to_string()));
    }
}
