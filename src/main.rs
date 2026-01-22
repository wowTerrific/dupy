mod error;
mod formatter;
mod grouper;
mod hasher;
mod scanner;

use std::env;
use std::path::PathBuf;
use std::process;

use error::{DupyError, Result};
use formatter::format_table;
use grouper::find_duplicates;
use scanner::walk_directory;

fn parse_args() -> Result<PathBuf> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return Err(DupyError::InvalidArguments(
            "Usage: dupy <directory>".to_string(),
        ));
    }

    let path = PathBuf::from(&args[1]);

    if !path.exists() {
        return Err(DupyError::InvalidPath(path));
    }

    if !path.is_dir() {
        return Err(DupyError::InvalidArguments(
            "Path must be a directory".to_string(),
        ));
    }

    Ok(path)
}

fn run() -> Result<()> {
    let directory = parse_args()?;

    // Scan directory for all files
    let files = walk_directory(&directory)?;

    // Find duplicates using 3-stage algorithm
    let duplicates = find_duplicates(files)?;

    // Format and print results
    format_table(&duplicates);

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
