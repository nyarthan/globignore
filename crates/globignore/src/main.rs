mod glob;

use std::{
    convert::Infallible, fmt::Display, fs::File, io::BufReader, ops::Deref, path::PathBuf,
    str::FromStr,
};

use clap::{Parser, ValueEnum};

use gitignore::parser::{GitignoreEntry, ParseError};
use glob::convert_to_globs;
use serde::Serialize;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, default_value_t = Cwd::default())]
    cwd: Cwd,
    #[arg(short, long, default_value_t = Format::Json)]
    format: Format,
}

#[derive(ValueEnum, Clone)]
enum Format {
    Yaml,
    Json,
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Json => "json",
                Self::Yaml => "yaml",
            }
        )
    }
}

#[derive(Clone)]
struct Cwd(PathBuf);

impl Display for Cwd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.to_str().expect("convert path to UTF-8 string")
        )
    }
}

impl Default for Cwd {
    fn default() -> Self {
        Self(std::env::current_dir().expect("retrieve current dir"))
    }
}

impl FromStr for Cwd {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(PathBuf::from_str(s)?))
    }
}

impl Deref for Cwd {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize)]
struct Output(Vec<String>);

fn main() {
    let Cli { cwd, format } = Cli::parse();

    let gitignore_file = File::open(cwd.join(".gitignore")).unwrap();
    let parser = gitignore::parser::Parser::new(BufReader::new(gitignore_file));
    let entries: Vec<GitignoreEntry> = parser
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            Err(_) => None,
        })
        .collect();

    let globs: Vec<String> = convert_to_globs(entries)
        .into_iter()
        .map(|glob| glob.pattern)
        .collect();
    let output = match format {
        Format::Json => serde_json::to_string(&Output(globs)).unwrap(),
        Format::Yaml => serde_yaml_ng::to_string(&Output(globs)).unwrap(),
    };

    println!("{output}");
}
