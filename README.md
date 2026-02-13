# dupy

A fast, zero-dependency Rust CLI tool for finding duplicate files in a directory.

## Overview

`dupy` recursively scans a specified directory and identifies duplicate files using an efficient 3-stage algorithm. Progress is reported to stderr by default, and results go to stdout — making it easy to redirect output for further processing.

## Features

- **Zero Dependencies**: Built using only the Rust standard library
- **Efficient Algorithm**: 3-stage duplicate detection minimizes I/O operations
  - Stage 1: Group by file size (cheap operation)
  - Stage 2: Quick hash of first 8KB (fast I/O filter)
  - Stage 3: Byte-by-byte verification (only on likely duplicates)
- **Name-Only Mode** (`--names`/`-n`): First-pass scan by filename with zero file I/O — fast and memory-efficient for large directories
- **Sorted Output**: Duplicate groups sorted by wasted space (largest first)
- **Size Filter** (`--min-size`): Skip files below a size threshold
- **Progress Reporting**: Scan progress printed to stderr by default; suppress with `--quiet`
- **CSV Output** (`--format csv`): Machine-readable output for scripting and spreadsheets
- **Exclude Patterns** (`--exclude`): Glob patterns to skip files by name
- **Junk File Defaults**: Common junk files (`Thumbs.db`, `.DS_Store`, `desktop.ini`, `~$*`, `*.tmp`, `*.lnk`) excluded by default; override with `--include-junk`
- **Memory Efficient**: Uses buffered reading with 8KB chunks — never loads entire files into memory
- **Graceful Error Handling**: Skips inaccessible files and continues scanning

## Building

```bash
cargo build --release
```

The compiled binary will be located at `./target/release/dupy`.

## Usage

```bash
dupy [--names|-n] [--min-size <size>] [--quiet|-q]
     [--format plain|csv] [--exclude <pattern>]...
     [--include-junk] <directory>
```

| Flag | Description |
|------|-------------|
| *(none)* | Full 3-stage content comparison (default) |
| `--names`, `-n` | Name-only mode: groups files by filename, no hashing or file I/O |
| `--min-size <size>` | Skip files smaller than `<size>` (e.g. `1MB`, `500KB`, `1048576`) |
| `--quiet`, `-q` | Suppress progress output on stderr |
| `--format plain\|csv` | Output format: `plain` (default) or `csv` |
| `--exclude <pattern>` | Exclude files matching glob pattern (repeatable) |
| `--include-junk` | Disable built-in junk file exclusions |

### Size suffixes

`--min-size` accepts a number with an optional suffix: `B`, `KB`/`K`, `MB`/`M`, `GB`/`G`.
Raw byte counts also work: `--min-size 1048576`.

### Examples

Find duplicates in a directory (full content check):
```bash
./target/release/dupy ~/Documents
```

**First-pass / memory-reduced scan using `--names`:**
```bash
./target/release/dupy --names ~/Documents
# or
./target/release/dupy -n ~/Documents
```

Name mode scans only filenames — no file reads, no hashing, no byte comparison. It is
significantly faster and uses far less memory than the default mode, making it a good
first pass on large or remote directories to spot obvious candidates before running a
full content scan.

> **Note:** Name matches are not guaranteed to be content duplicates. Use the default
> mode (or pipe name-mode results) to confirm actual duplicates.

Only scan files 1 MB or larger:
```bash
./target/release/dupy --min-size 1MB ~/Downloads
```

Export results as CSV:
```bash
./target/release/dupy --format csv ~/share > report.csv
```

Suppress progress output (stdout only):
```bash
./target/release/dupy --quiet ~/share
./target/release/dupy ~/share 2>/dev/null
```

Exclude backup and log files:
```bash
./target/release/dupy --exclude "*.bak" --exclude "*.log" ~/share
```

Include junk files that are normally skipped:
```bash
./target/release/dupy --include-junk ~/share
```

Combined example for a network share audit:
```bash
./target/release/dupy --min-size 1MB --format csv --exclude "*.bak" --quiet ~/share > report.csv
```

## Example Output

### Default mode (content duplicates)

```
Scanned 1042 files.
Checking 18 size groups with potential duplicates...
Verifying 7 hash groups...
Done.
Duplicate Files Report
======================

Group 1 (3 files, 1.2 MB each, 2.4 MB wasted):
  /home/user/Documents/report.pdf
  /home/user/Downloads/report.pdf
  /home/user/Backup/report.pdf

Group 2 (2 files, 512 KB each, 512 KB wasted):
  /home/user/Pictures/photo1.jpg
  /home/user/Pictures/backup/photo1.jpg

Summary:
--------
Total duplicate groups: 2
Total duplicate files: 5 (of which 3 are redundant copies)
Total wasted space: 2.9 MB
```

### `--format csv` mode

```
group_id,size_bytes,size_human,wasted_bytes,wasted_human,file_path
1,1258291,1.20 MB,2516582,2.40 MB,/home/user/Documents/report.pdf
1,1258291,1.20 MB,2516582,2.40 MB,/home/user/Downloads/report.pdf
1,1258291,1.20 MB,2516582,2.40 MB,/home/user/Backup/report.pdf
2,524288,512.00 KB,524288,512.00 KB,/home/user/Pictures/photo1.jpg
2,524288,512.00 KB,524288,512.00 KB,/home/user/Pictures/backup/photo1.jpg
```

### `--names` mode (filename duplicates)

```
Duplicate Names Report
======================

Group 1 - "photo1.jpg" (2 files):
  /home/user/Pictures/photo1.jpg
  /home/user/Pictures/backup/photo1.jpg

Group 2 - "report.pdf" (3 files):
  /home/user/Documents/report.pdf
  /home/user/Downloads/report.pdf
  /home/user/Backup/report.pdf

Summary:
--------
Total groups with shared names: 2
Total files involved: 5
```

## How It Works

### 3-Stage Duplicate Detection

1. **Size Grouping**: Files are grouped by size. Groups with only one file are immediately eliminated (no duplicates possible).

2. **Quick Hash**: For each size group with 2+ files, the first 8KB of each file is hashed. Files are grouped by hash value. Groups with only one file are eliminated.

3. **Byte-by-Byte Verification**: For remaining candidates, files are compared chunk-by-chunk (8KB chunks) using buffered reading. Only files with identical content are marked as duplicates.

Results are sorted by wasted space (largest first) so the most impactful duplicates appear at the top.

### Glob Pattern Matching

Exclude patterns use a simple built-in glob engine (no external crates):
- `*` matches any sequence of characters (including empty)
- `?` matches any single character
- Patterns match the filename only, not the full path
- Matching is case-sensitive on all platforms

Built-in junk excludes (active unless `--include-junk` is passed):
`Thumbs.db`, `.DS_Store`, `desktop.ini`, `~$*`, `*.tmp`, `*.lnk`

## Project Structure

```
src/
├── main.rs       # Entry point, CLI parsing, orchestration
├── scanner.rs    # Directory traversal logic with exclusion filtering
├── grouper.rs    # 3-stage duplicate detection algorithm with progress reporting
├── hasher.rs     # Quick hash + byte-by-byte comparison
├── formatter.rs  # Plain text and CSV output formatting
├── glob.rs       # Glob pattern matching (*, ?)
└── error.rs      # Custom error types
```

## Error Handling

- **Fatal errors**: Invalid directory path, inaccessible target directory
- **Non-fatal errors**: Permission denied on specific files, symlinks, corrupted files
  - The tool skips the problematic file and continues scanning
  - Ensures maximum coverage even in directories with problematic files

## Development

### Run Tests

```bash
# Create test data
mkdir -p test_data/{unique,duplicates/nested}
echo "unique content 1" > test_data/unique/file1.txt
echo "duplicate content" > test_data/duplicates/doc1.txt
echo "duplicate content" > test_data/duplicates/doc2.txt
echo "duplicate content" > test_data/duplicates/nested/doc3.txt

# Run dupy
./target/release/dupy test_data

# CSV output
./target/release/dupy --format csv test_data

# Exclude patterns
./target/release/dupy --exclude "*.txt" test_data

# Min size filter
./target/release/dupy --min-size 1KB test_data
```

### Code Quality

```bash
# Run clippy
cargo clippy

# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

## Future Implementation Ideas

- **Hidden Files**: Add `--no-hidden` flag to skip hidden files (starting with `.`)
- **Output Formats**: `--format json` for programmatic processing
- **Parallel Processing**: Add parallel file hashing for large directories
- **Interactive Mode**: Interactive CLI to preview and delete duplicates
- **Follow Symlinks**: Add `--follow-symlinks` flag (currently skipped)
- **Hash Caching**: Cache file hashes to speed up repeated scans
- **Deduplication Actions**:
  - `--delete` to automatically remove duplicates
  - `--hardlink` to replace duplicates with hardlinks
  - `--move-to` to move duplicates to a specified directory
- **Content Preview**: Show first few bytes or lines of duplicate files for verification
- **Incremental Scanning**: Store results and only scan changed files on subsequent runs
- **Filtering by File Type**: Add `--type` flag to only scan specific file extensions
- **Statistics Mode**: Add `--stats-only` for summary without file listing

## License

This project is provided as-is for educational and personal use.
