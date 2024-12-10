use std::{
    fs::File,
    io::{self, Read as _, Write as _},
    path::{Path, PathBuf},
};

use clap::Parser;
use seesaw::{Destination, Trait};

/// Generate a trait from `extern "C"` blocks,
/// typically emitted by bindgen.
#[derive(Parser)]
#[command(name = env!("CARGO_BIN_NAME"))]
struct Args {
    /// The name of the trait.
    name: String,
    /// Regexes of function names to include.
    /// By default, all functions are allowed.
    #[arg(short, long)]
    allow: Vec<String>,
    /// Regexes of function names to exclude.
    #[arg(short, long)]
    block: Vec<String>,
    /// Path the the input file.
    /// If absent or `-`, read from stdin.
    bindings: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let Args {
        name,
        allow,
        block,
        bindings,
    } = Args::parse();

    let mut s = String::new();
    match bindings {
        Some(it) if it.as_path() == Path::new("-") => stdin(&mut s)?,
        Some(openme) => File::open(openme)?.read_to_string(&mut s)?,
        None => stdin(&mut s)?,
    };

    seesaw::seesaw(
        Trait::new(name).allow_all(allow).block_all(block),
        s,
        Destination::Writer(Box::new(io::stdout())),
    )
}

fn stdin(s: &mut String) -> io::Result<usize> {
    writeln!(io::stderr(), "seesaw: reading from stdin...")?;
    io::stdin().read_to_string(s)
}

#[test]
fn doc() {
    expect_test::expect_file!["README.md"].assert_eq(&clap_doc::markdown_for::<Args>());
}
