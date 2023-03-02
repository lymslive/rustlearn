#![allow(unused)]

use clap::Parser;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    pattern: String,
    /// The path to the file to read
    path: std::path::PathBuf,
}

fn main_v1() {
    println!("Hello, world!");
    let pattern = std::env::args().nth(1).expect("no pattern given");
    let path = std::env::args().nth(2).expect("no path given");
    let args = Cli {
        pattern: pattern,
        path: std::path::PathBuf::from(path),
    };
}

fn main_v2() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    println!("#1 {}, #2 {}", args.pattern, args.path.display());

    let content = std::fs::read_to_string(&args.path)?; //.expect("could not read file");
    for line in content.lines() {
        if line.contains(&args.pattern) {
            println!("{}", line);
        }
    }

    return Ok(());
}

#[derive(Debug)]
struct CustomError(String);

fn main_v3() -> Result<(), CustomError> {
    let args = Cli::parse();
    println!("#1 {}, #2 {}", args.pattern, args.path.display());

    let content = std::fs::read_to_string(&args.path)
        .map_err(|err| CustomError(format!("Error reading `{}`: {}", args.path.to_string_lossy(), err)))?;
    for line in content.lines() {
        if line.contains(&args.pattern) {
            println!("{}", line);
        }
    }

    return Ok(());
}

use anyhow::{Context, Result};
use log::{info, warn};

fn main() -> Result<()> {
    env_logger::init();
    let args = Cli::parse();
    info!("#1 {}, #2 {}", args.pattern, args.path.display());

    let content = std::fs::read_to_string(&args.path)
        .with_context(|| format!("could not read file `{}`", args.path.display()))?;
    find_matches(&content, &args.pattern, &mut std::io::stdout());

    Ok(())
}

fn find_matches(content: &str, pattern: &str, mut writer: impl std::io::Write) {
    for line in content.lines() {
        if line.contains(&pattern) {
            writeln!(writer, "{}", line);
        }
    }
}

#[test]
fn find_a_match() {
    let mut result = Vec::new();
    find_matches("lorem ipsum\ndolor sit amet", "lorem", &mut result);
    assert_eq!(result, b"lorem ipsum\n");
}
