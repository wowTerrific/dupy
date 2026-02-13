# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

dupy is a zero-dependency Rust CLI tool for finding duplicate files using a 3-stage detection algorithm. The project is intentionally minimal - it uses **only the Rust standard library** with no external crates.

## Core Architecture

### 3-Stage Duplicate Detection Pipeline

The duplicate detection flows through three stages in `grouper.rs`:

1. **Size Grouping** (`group_by_size`): Groups files by size, eliminating unique sizes
2. **Quick Hash** (`group_by_hash`): Hashes first 8KB of each file to filter likely duplicates
3. **Byte-by-Byte Verification** (`verify_duplicates`): Full content comparison using buffered I/O

Results are sorted descending by wasted space (`size * (count - 1)`) after verification.

### Module Responsibilities

- **error.rs**: Custom error types with automatic conversion from `std::io::Error`
- **glob.rs**: Built-in glob matching (`*`, `?`) with no external deps; patterns match filename only
- **scanner.rs**: Recursive directory traversal that gracefully skips inaccessible files/symlinks
  - `walk_directory(path, user_excludes, include_junk)`: returns `Vec<FileInfo>` (path + size) for content mode; applies built-in junk excludes and user `--exclude` patterns
  - `walk_directory_names()`: returns `Vec<(OsString, PathBuf)>` (filename + path) for name mode; no exclusion filtering
- **hasher.rs**:
  - `quick_hash()`: Computes hash of first 8KB using bit rotation
  - `files_identical()`: Chunk-by-chunk comparison (8KB chunks)
- **grouper.rs**: Orchestrates duplicate detection for both modes
  - `find_duplicates(files, min_size, verbose)`: 3-stage content pipeline (size → quick hash → byte comparison); filters by `min_size` before grouping; progress messages to stderr when `verbose`
  - `find_name_duplicates()`: name-only grouping with no file I/O
- **formatter.rs**: Human-readable output with byte formatting (B/KB/MB/GB)
  - `format_table()`: content-mode report sorted by wasted space
  - `format_name_table()`: name-mode report grouped by shared filename
  - `format_csv()`: CSV output (group_id, size_bytes, size_human, wasted_bytes, wasted_human, file_path)
- **main.rs**: CLI argument parsing, mode/format dispatch, error handling (exits with code 1 on error)

### Key Constraints

- **No external dependencies**: Keep `Cargo.toml` dependencies section empty
- **Memory efficiency**: Never load entire files into memory - always use 8KB buffered chunks
- **Error resilience**: Non-fatal errors (permission denied, corrupted files) should log and continue, not crash

## CLI Flags

```
dupy [--names|-n] [--min-size <size>] [--quiet|-q]
     [--format plain|csv] [--exclude <pattern>]...
     [--include-junk] <directory>
```

| Flag | Default | Description |
|------|---------|-------------|
| `--names`, `-n` | off | Name-only mode (no file I/O) |
| `--min-size <size>` | `0` | Skip files below threshold; suffix: B/KB/MB/GB |
| `--quiet`, `-q` | off | Suppress stderr progress |
| `--format plain\|csv` | `plain` | Output format |
| `--exclude <pattern>` | none | Glob pattern to exclude (repeatable) |
| `--include-junk` | off | Disable built-in junk excludes |

Built-in junk excludes: `Thumbs.db`, `.DS_Store`, `desktop.ini`, `~$*`, `*.tmp`, `*.lnk`

Progress is on by default (stderr); suppress with `--quiet`. "Scanned N files." reflects post-exclusion count.

## Common Commands

```bash
# Build release binary
cargo build --release

# Run on a directory (full content check)
./target/release/dupy <directory>

# Run name-only first-pass (no file I/O, faster, lower memory)
./target/release/dupy --names <directory>
./target/release/dupy -n <directory>

# Min size filter
./target/release/dupy --min-size 1MB <directory>

# CSV output
./target/release/dupy --format csv <directory> > report.csv

# Quiet (stdout only)
./target/release/dupy --quiet <directory>

# Exclude patterns
./target/release/dupy --exclude "*.bak" --exclude "*.log" <directory>

# Code quality checks
cargo clippy           # Lint
cargo fmt              # Format code
cargo fmt --check      # Check formatting without changes

# Create test data
mkdir -p test_data/{unique,duplicates/nested}
echo "unique content 1" > test_data/unique/file1.txt
echo "duplicate content" > test_data/duplicates/doc1.txt
echo "duplicate content" > test_data/duplicates/doc2.txt
./target/release/dupy test_data
./target/release/dupy --names test_data
```

## Making Changes

### Adding New Features

When adding CLI flags or new functionality:

1. **Update `main.rs`** for argument parsing — use the index-based `while` loop when a flag consumes the next token
2. **Modify the pipeline** in `grouper.rs` if changing duplicate detection logic
3. **Update `formatter.rs`** if changing output format
4. **Keep it zero-dependency** - no external crates allowed

### Performance Considerations

- The 8KB chunk size (`CHUNK_SIZE` in `hasher.rs`) balances I/O efficiency and memory usage
- HashMap groupings use `.or_default()` for cleaner code (clippy preference)
- Early bailouts prevent unnecessary work (skip size groups with 1 file)
- `min_size` filtering happens before size grouping to reduce HashMap work

### Error Handling Pattern

Use the custom `Result<T>` type alias from `error.rs` throughout. The `?` operator automatically converts `std::io::Error` via the `From` trait implementation. Fatal errors return early; non-fatal errors (file access issues) skip the file and continue.
