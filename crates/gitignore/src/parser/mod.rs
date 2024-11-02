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
    use super::*;

    mod util {
        use super::*;

        pub fn from_str(s: &str) -> Parser<io::BufReader<&[u8]>> {
            Parser::new(io::BufReader::new(s.as_bytes()))
        }
    }

    #[test]
    fn parse_string() {
        let input = "root.txt\n#comment\ndir/";
        let parser = util::from_str(input);
        let entries: Vec<_> = parser.collect();

        assert_eq!(entries.len(), 3);
        assert!(matches!(entries[0], Ok(GitignoreEntry::Pattern(_))));
        assert!(matches!(entries[1], Ok(GitignoreEntry::Comment(_))));
        assert!(matches!(entries[2], Ok(GitignoreEntry::Pattern(_))));
    }
}
