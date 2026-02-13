mod error;
mod formatter;
mod glob;
mod grouper;
mod hasher;
mod scanner;

use std::env;
use std::path::PathBuf;
use std::process;

use error::{DupyError, Result};
use formatter::{format_csv, format_name_table, format_table};
use grouper::{find_duplicates, find_name_duplicates};
use scanner::walk_directory;

enum Mode {
    Content,
    Names,
}

enum OutputFormat {
    Plain,
    Csv,
}

struct Args {
    directory: PathBuf,
    mode: Mode,
    min_size: u64,
    quiet: bool,
    format: OutputFormat,
    excludes: Vec<String>,
    include_junk: bool,
}

fn parse_size(s: &str) -> Result<u64> {
    let split_pos = s
        .char_indices()
        .find(|(_, c)| !c.is_ascii_digit())
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    let (num_str, suffix) = s.split_at(split_pos);
    if num_str.is_empty() {
        return Err(DupyError::InvalidArguments(format!(
            "Invalid size '{}': no numeric prefix",
            s
        )));
    }
    let num: u64 = num_str.parse().map_err(|_| {
        DupyError::InvalidArguments(format!("Invalid size '{}': numeric part too large", s))
    })?;
    let multiplier: u64 = match suffix.to_uppercase().as_str() {
        "" | "B" => 1,
        "KB" | "K" => 1024,
        "MB" | "M" => 1024 * 1024,
        "GB" | "G" => 1024 * 1024 * 1024,
        other => {
            return Err(DupyError::InvalidArguments(format!(
                "Unknown size suffix '{}' (use B, KB, MB, GB)",
                other
            )))
        }
    };
    Ok(num.saturating_mul(multiplier))
}

fn parse_args() -> Result<Args> {
    let args: Vec<String> = env::args().collect();
    let mut mode = Mode::Content;
    let mut dir_arg: Option<String> = None;
    let mut min_size: u64 = 0;
    let mut quiet = false;
    let mut format = OutputFormat::Plain;
    let mut excludes: Vec<String> = Vec::new();
    let mut include_junk = false;

    const USAGE: &str = "Usage: dupy [--names|-n] [--min-size <size>] [--quiet|-q]\n\
         \x20       [--format plain|csv] [--exclude <pattern>]...\n\
         \x20       [--include-junk] <directory>";

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--names" | "-n" => mode = Mode::Names,
            "--quiet" | "-q" => quiet = true,
            "--include-junk" => include_junk = true,
            "--min-size" => {
                i += 1;
                if i >= args.len() {
                    return Err(DupyError::InvalidArguments(
                        "--min-size requires a value".to_string(),
                    ));
                }
                min_size = parse_size(&args[i])?;
            }
            "--format" => {
                i += 1;
                if i >= args.len() {
                    return Err(DupyError::InvalidArguments(
                        "--format requires a value (plain or csv)".to_string(),
                    ));
                }
                format = match args[i].as_str() {
                    "plain" => OutputFormat::Plain,
                    "csv" => OutputFormat::Csv,
                    other => {
                        return Err(DupyError::InvalidArguments(format!(
                            "Unknown format '{}' (use plain or csv)",
                            other
                        )))
                    }
                };
            }
            "--exclude" => {
                i += 1;
                if i >= args.len() {
                    return Err(DupyError::InvalidArguments(
                        "--exclude requires a pattern".to_string(),
                    ));
                }
                excludes.push(args[i].clone());
            }
            other => {
                if dir_arg.is_some() {
                    return Err(DupyError::InvalidArguments(USAGE.to_string()));
                }
                dir_arg = Some(other.to_string());
            }
        }
        i += 1;
    }

    let dir_str = dir_arg.ok_or_else(|| DupyError::InvalidArguments(USAGE.to_string()))?;
    let path = PathBuf::from(&dir_str);
    if !path.exists() {
        return Err(DupyError::InvalidPath(path));
    }
    if !path.is_dir() {
        return Err(DupyError::InvalidArguments(
            "Path must be a directory".to_string(),
        ));
    }
    Ok(Args {
        directory: path,
        mode,
        min_size,
        quiet,
        format,
        excludes,
        include_junk,
    })
}

fn run() -> Result<()> {
    let args = parse_args()?;
    match args.mode {
        Mode::Content => {
            let files = walk_directory(&args.directory, &args.excludes, args.include_junk)?;
            if !args.quiet {
                eprintln!("Scanned {} files.", files.len());
            }
            let duplicates = find_duplicates(files, args.min_size, !args.quiet)?;
            match args.format {
                OutputFormat::Plain => format_table(&duplicates),
                OutputFormat::Csv => format_csv(&duplicates),
            }
        }
        Mode::Names => {
            let groups = find_name_duplicates(&args.directory)?;
            format_name_table(&groups);
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
