use std::{convert::Infallible, fmt::Display, ops::Deref, path::PathBuf, str::FromStr};

use clap::{Parser, ValueEnum};

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

fn main() {
    let Cli { cwd, format } = Cli::parse();

    println!("{} {}", cwd, format);
}
