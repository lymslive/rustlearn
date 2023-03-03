use clap::{CommandFactory, Parser};
use is_terminal::IsTerminal as _;
use std::{
    fs::File,
    io::{stdin, BufRead, BufReader},
    path::PathBuf
};

use exitcode;
// extern crate exitcode; // also ok

/// Count the number of words in a file.
#[derive(Parser)]
#[command(arg_required_else_help = true)]
struct Cli {
    /// The path to the file to read, use - to read from stdin (must not be a tty)
    file: PathBuf,
}

fn main() {
    let args = Cli::parse();
    let word_count;
    let mut file = args.file;

    if file == PathBuf::from("-") {
        if stdin().is_terminal() {
            Cli::command().print_help().unwrap();
            ::std::process::exit(exitcode::USAGE);
        }
        file = PathBuf::from("<stdin>");
        word_count = words_in_buf_reader(BufReader::new(stdin().lock()));
    }
    else {
        word_count = words_in_buf_reader(BufReader::new(File::open(&file).unwrap()));
    }
    
    println!("Words in {}: {}", file.to_string_lossy(), word_count);
}

fn words_in_buf_reader<R: BufRead>(buf_reader: R) -> usize {
    let mut word_count = 0;

    for line in buf_reader.lines() {
        word_count += line.unwrap().split(' ').count();
    }
    word_count
}
