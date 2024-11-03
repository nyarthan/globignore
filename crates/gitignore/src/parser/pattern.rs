use super::ParseError;

#[derive(Debug)]
pub struct Pattern {
    pub raw: String,
    pub is_negated: bool,
    pub has_leading_slash: bool,
    pub has_trailing_slash: bool,
    pub was_escaped: bool,
    pub segments: Vec<Segment>,
}

#[derive(Debug, PartialEq)]
pub enum Segment {
    Literal(String),
    Wildcard,
    SingleWildcard,
    DoubleWildcard,
    Separator,
    CharacterClass {
        negated: bool,
        chars: Vec<char>,
        ranges: Vec<(char, char)>,
    },
}

impl Pattern {
    pub fn new(
        raw: String,
        is_negated: bool,
        has_leading_slash: bool,
        has_trailing_slash: bool,
        was_escaped: bool,
    ) -> Result<Self, ParseError> {
        let segments = Pattern::parse_segments(raw.as_str())?;
        Ok(Self {
            raw,
            is_negated,
            has_leading_slash,
            has_trailing_slash,
            was_escaped,
            segments,
        })
    }
    pub fn parse_segments(raw: &str) -> Result<Vec<Segment>, ParseError> {
        let mut segments = Vec::new();
        let mut chars = raw.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '?' => segments.push(Segment::SingleWildcard),
                '*' => {
                    if chars.peek() == Some(&'*') {
                        chars.next();

                        let is_double_wildcard = match (segments.last(), chars.peek()) {
                            (Some(Segment::Separator), Some('/')) => true,
                            (Some(Segment::Separator), _) => true,
                            (_, Some('/')) => true,
                            _ => false,
                        };

                        if is_double_wildcard {
                            segments.push(Segment::DoubleWildcard);
                        } else {
                            segments.push(Segment::Wildcard);
                            segments.push(Segment::Wildcard);
                        }
                    } else {
                        segments.push(Segment::Wildcard)
                    }
                }
                '/' => segments.push(Segment::Separator),
                '\\' => {
                    if let Some(next) = chars.next() {
                        segments.push(Segment::Literal(next.to_string()));
                    }
                }
                c => {
                    let mut literal = String::new();
                    literal.push(c);
                    while let Some(&next) = chars.peek() {
                        if "?*[/\\".contains(next) {
                            break;
                        }
                        literal.push(chars.next().unwrap());
                    }
                    segments.push(Segment::Literal(literal));
                }
            }
        }

        Ok(segments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_segments(pattern: &str, expected: Vec<Segment>) {
        let result = Pattern::parse_segments(pattern).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_literal() {
        assert_segments("hello.txt", vec![Segment::Literal("hello.txt".to_string())]);
    }

    #[test]
    fn test_single_wildcard() {
        assert_segments(
            "hello?.txt",
            vec![
                Segment::Literal("hello".to_string()),
                Segment::SingleWildcard,
                Segment::Literal(".txt".to_string()),
            ],
        );
    }

    #[test]
    fn test_wildcard() {
        assert_segments(
            "*.txt",
            vec![Segment::Wildcard, Segment::Literal(".txt".to_string())],
        );
    }

    #[test]
    fn test_double_star() {
        assert_segments(
            "**/build",
            vec![
                Segment::DoubleWildcard,
                Segment::Separator,
                Segment::Literal("build".to_string()),
            ],
        );
    }

    #[test]
    fn test_separators() {
        assert_segments(
            "src/main.rs",
            vec![
                Segment::Literal("src".to_string()),
                Segment::Separator,
                Segment::Literal("main.rs".to_string()),
            ],
        );
    }

    #[test]
    fn test_escaped_characters() {
        assert_segments(
            r"hello\.txt",
            vec![
                Segment::Literal("hello".to_string()),
                Segment::Literal(".".to_string()),
                Segment::Literal("txt".to_string()),
            ],
        );
    }

    #[test]
    fn test_complex_pattern() {
        assert_segments(
            "src/**/test/*.rs",
            vec![
                Segment::Literal("src".to_string()),
                Segment::Separator,
                Segment::DoubleWildcard,
                Segment::Separator,
                Segment::Literal("test".to_string()),
                Segment::Separator,
                Segment::Wildcard,
                Segment::Literal(".rs".to_string()),
            ],
        );
    }

    #[test]
    fn test_pattern_new() {
        let pattern = Pattern::new("src/**/*.rs".to_string(), false, false, false, false).unwrap();

        assert_eq!(pattern.raw, "src/**/*.rs");
        assert!(!pattern.is_negated);
        assert!(!pattern.has_leading_slash);
        assert!(!pattern.has_trailing_slash);
        assert!(!pattern.was_escaped);

        assert_eq!(
            pattern.segments,
            vec![
                Segment::Literal("src".to_string()),
                Segment::Separator,
                Segment::DoubleWildcard,
                Segment::Separator,
                Segment::Wildcard,
                Segment::Literal(".rs".to_string()),
            ]
        );
    }

    #[test]
    fn test_consecutive_separators() {
        assert_segments(
            "a//b",
            vec![
                Segment::Literal("a".to_string()),
                Segment::Separator,
                Segment::Separator,
                Segment::Literal("b".to_string()),
            ],
        );
    }

    #[test]
    fn test_trailing_escape() {
        // Make sure we handle a trailing backslash correctly
        assert_segments("hello\\", vec![Segment::Literal("hello".to_string())]);
    }

    #[test]
    fn test_empty_pattern() {
        assert_segments("", vec![]);
    }

    #[test]
    fn test_only_wildcards() {
        assert_segments(
            "***?**",
            vec![
                Segment::Wildcard,
                Segment::Wildcard,
                Segment::Wildcard,
                Segment::SingleWildcard,
                Segment::Wildcard,
                Segment::Wildcard,
            ],
        );
    }

    #[test]
    fn test_only_separators() {
        assert_segments(
            "///",
            vec![Segment::Separator, Segment::Separator, Segment::Separator],
        );
    }

    #[test]
    fn test_escaped_special_chars() {
        assert_segments(
            r"\?\*\[\/\\",
            vec![
                Segment::Literal("?".to_string()),
                Segment::Literal("*".to_string()),
                Segment::Literal("[".to_string()),
                Segment::Literal("/".to_string()),
                Segment::Literal("\\".to_string()),
            ],
        );
    }
}
