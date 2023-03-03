// from cli book 2.6
// https://rust-cli.github.io/book/in-depth/docs.html
// used with build.rs

use clap::Parser;

#[derive(Parser)]
pub struct Head {
    /// file to load
    pub file: std::path::PathBuf,
    /// how many line to print
    #[arg(short = 'n', default_value = "5")]
    pub count: usize,
}
