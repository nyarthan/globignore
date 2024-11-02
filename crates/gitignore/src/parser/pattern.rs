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

#[derive(Debug)]
pub enum Segment {
    Literal(String),
    SingleWildcard,
    Wildcard,
    DoubleStar,
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
                        segments.push(Segment::DoubleStar);
                    } else {
                        segments.push(Segment::Wildcard);
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
