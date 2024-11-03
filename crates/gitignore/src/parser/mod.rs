mod pattern;

use std::io::{self, BufRead};

use pattern::Pattern;

#[derive(Debug)]
pub enum GitignoreEntry {
    Pattern(Pattern),
    Blank,
    Comment(String),
}

#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    InvalidPattern,
}

pub struct Parser<R> {
    reader: R,
    buffer: String,
}

impl<R: BufRead> Parser<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: String::with_capacity(1024),
        }
    }

    fn parse_next_line(&mut self) -> Option<Result<GitignoreEntry, ParseError>> {
        self.buffer.clear();
        match self.reader.read_line(&mut self.buffer) {
            Ok(0) => None,
            Ok(_) => Some(self.parse_line()),
            Err(e) => Some(Err(ParseError::Io(e))),
        }
    }

    fn parse_line(&self) -> Result<GitignoreEntry, ParseError> {
        let line = self.buffer.trim_end();

        if line.is_empty() {
            return Ok(GitignoreEntry::Blank);
        }

        // do I need the second check?
        if line.starts_with('#') && !line.starts_with(r"\#") {
            return Ok(GitignoreEntry::Comment(line[1..].to_string()));
        }

        let (pattern, was_escaped) = if line.starts_with('\\') {
            (&line[1..], true)
        } else {
            (line, false)
        };

        let (pattern, is_negated) = if pattern.starts_with('!') && !was_escaped {
            (&pattern[1..], true)
        } else {
            (pattern, false)
        };

        let has_leading_slash = pattern.starts_with('/');
        let has_trailing_slash = pattern.ends_with('/') && !pattern.ends_with(r"\/");

        Ok(GitignoreEntry::Pattern(Pattern::new(
            pattern.to_string(),
            is_negated,
            has_leading_slash,
            has_trailing_slash,
            was_escaped,
        )?))
    }
}

impl<R: BufRead> Iterator for Parser<R> {
    type Item = Result<GitignoreEntry, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next_line()
    }
}

#[cfg(test)]
mod tests {
    use dedent::dedent;

    use super::*;
    use pattern::Segment::*;
    use GitignoreEntry::*;

    mod util {
        use super::*;

        pub fn from_str(s: &str) -> Parser<io::BufReader<&[u8]>> {
            Parser::new(io::BufReader::new(s.as_bytes()))
        }

        pub fn assert_pattern(pattern: &pattern::Pattern, expected: &Vec<pattern::Segment>) {
            assert_eq!(&pattern.segments, expected);
        }
    }

    #[test]
    fn parse_string() {
        let input = dedent!(
            "
            root.txt
            #comment
            dir/
        "
        );
        let parser = util::from_str(input.as_str());
        let entries: Vec<_> = parser.collect();

        assert_eq!(entries.len(), 3);
        assert!(matches!(entries[0], Ok(Pattern(_))));
        assert!(matches!(entries[1], Ok(Comment(_))));
        assert!(matches!(entries[2], Ok(Pattern(_))));
    }

    #[test]
    fn patterns() {
        let input = dedent!(
            "
            /foo
            /foo/*
            /foo/**
            /foo/**/*.bar
            */baz
            **/baz
            foo/**/bar
            foo/?/bar
            **?**
"
        );
        let parser = util::from_str(input.as_str());
        let entries: Vec<_> = parser.collect();
        let patterns: Vec<pattern::Pattern> = entries
            .into_iter()
            .filter_map(|entry| match entry {
                Ok(GitignoreEntry::Pattern(p)) => Some(p),
                _ => None,
            })
            .collect();

        let expected = vec![
            vec![Separator, Literal("foo".into())],
            vec![Separator, Literal("foo".into()), Separator, Wildcard],
            vec![Separator, Literal("foo".into()), Separator, DoubleWildcard],
            vec![
                Separator,
                Literal("foo".into()),
                Separator,
                DoubleWildcard,
                Separator,
                Wildcard,
                Literal(".bar".into()),
            ],
            vec![Wildcard, Separator, Literal("baz".into())],
            vec![DoubleWildcard, Separator, Literal("baz".into())],
            vec![
                Literal("foo".into()),
                Separator,
                DoubleWildcard,
                Separator,
                Literal("bar".into()),
            ],
            vec![
                Literal("foo".into()),
                Separator,
                SingleWildcard,
                Separator,
                Literal("bar".into()),
            ],
            vec![Wildcard, Wildcard, SingleWildcard, Wildcard, Wildcard],
        ];

        assert_eq!(patterns.len(), expected.len());

        for (pattern, expected_segments) in patterns.iter().zip(expected.iter()) {
            util::assert_pattern(pattern, expected_segments);
        }
    }
}
