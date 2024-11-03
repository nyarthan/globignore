use gitignore::parser::{
    pattern::{Pattern, Segment},
    GitignoreEntry,
};

#[derive(Debug, PartialEq)]
pub struct GlobPattern {
    pub pattern: String,
    pub is_negated: bool,
}

pub fn convert_to_globs(entries: Vec<GitignoreEntry>) -> Vec<GlobPattern> {
    entries
        .into_iter()
        .filter_map(|entry| match entry {
            GitignoreEntry::Pattern(pattern) => Some(pattern_to_glob(pattern)),
            _ => None,
        })
        .collect()
}

pub fn pattern_to_glob(pattern: Pattern) -> GlobPattern {
    let mut glob_str = String::new();

    if pattern.has_leading_slash {
        glob_str.push('.');
    } else {
        glob_str.push_str("**/");
    }

    for segment in pattern.segments {
        match segment {
            Segment::Literal(s) => glob_str.push_str(&s),
            Segment::Wildcard => glob_str.push('*'),
            Segment::SingleWildcard => glob_str.push('?'),
            Segment::DoubleWildcard => glob_str.push_str("**"),
            Segment::Separator => glob_str.push('/'),
            Segment::CharacterClass {
                negated,
                chars,
                ranges,
            } => {
                glob_str.push('[');
                if negated {
                    glob_str.push('^');
                }
                for c in chars {
                    glob_str.push(c);
                }
                for (start, end) in ranges {
                    glob_str.push(start);
                    glob_str.push('-');
                    glob_str.push(end);
                }
                glob_str.push(']')
            }
        }
    }

    if pattern.has_trailing_slash {
        glob_str.push_str("**");
    }

    GlobPattern {
        pattern: glob_str,
        is_negated: pattern.is_negated,
    }
}
