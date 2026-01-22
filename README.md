# dupy

A fast, zero-dependency Rust CLI tool for finding duplicate files in a directory.

## Overview

`dupy` recursively scans a specified directory and identifies duplicate files using an efficient 3-stage algorithm. It outputs results in a human-readable table format to stdout, making it easy to pipe the output for further processing.

## Features

- **Zero Dependencies**: Built using only the Rust standard library
- **Efficient Algorithm**: 3-stage duplicate detection minimizes I/O operations
  - Stage 1: Group by file size (cheap operation)
  - Stage 2: Quick hash of first 8KB (fast I/O filter)
  - Stage 3: Byte-by-byte verification (only on likely duplicates)
- **Memory Efficient**: Uses buffered reading with 8KB chunks - never loads entire files into memory
- **Graceful Error Handling**: Skips inaccessible files and continues scanning
- **Human-Readable Output**: Shows file sizes in B/KB/MB/GB with summary statistics
- **Pipeable Output**: Clean stdout format for integration with other tools

## Building

```bash
cargo build --release
```

The compiled binary will be located at `./target/release/dupy`.

## Usage

```bash
dupy <directory>
```

### Examples

Find duplicates in a directory:
```bash
./target/release/dupy ~/Documents
```

Find duplicates and save to a file:
```bash
./target/release/dupy ~/Downloads > duplicates.txt
```

Find duplicates and filter by group:
```bash
./target/release/dupy ~/Pictures | grep "Group"
```

## Example Output

```
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

## How It Works

### 3-Stage Duplicate Detection

1. **Size Grouping**: Files are grouped by size. Groups with only one file are immediately eliminated (no duplicates possible).

2. **Quick Hash**: For each size group with 2+ files, the first 8KB of each file is hashed. Files are grouped by hash value. Groups with only one file are eliminated.

3. **Byte-by-Byte Verification**: For remaining candidates, files are compared chunk-by-chunk (8KB chunks) using buffered reading. Only files with identical content are marked as duplicates.

This approach ensures that:
- Most files are eliminated at the cheap size-check stage
- The quick hash eliminates most false positives with minimal I/O
- Full verification is only performed on likely duplicates

## Project Structure

```
src/
├── main.rs       # Entry point, CLI parsing, orchestration
├── scanner.rs    # Directory traversal logic
├── grouper.rs    # 3-stage duplicate detection algorithm
├── hasher.rs     # Quick hash + byte-by-byte comparison
├── formatter.rs  # Table output formatting
└── error.rs      # Custom error types
```

## Error Handling

- **Fatal errors**: Invalid directory path, inaccessible target directory
- **Non-fatal errors**: Permission denied on specific files, symlinks, corrupted files
  - The tool logs warnings and continues scanning
  - Ensures maximum coverage even in directories with problematic files

## Development

### Run Tests

```bash
# Create test data
mkdir -p test_data/{unique,duplicates/nested}
echo "unique content 1" > test_data/unique/file1.txt
echo "unique content 2" > test_data/unique/file2.txt
echo "duplicate content" > test_data/duplicates/doc1.txt
echo "duplicate content" > test_data/duplicates/doc2.txt
echo "duplicate content" > test_data/duplicates/nested/doc3.txt

# Run dupy
./target/release/dupy test_data
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

- **Size Filtering**: Add `--min-size` flag to skip files smaller than a threshold
- **Hidden Files**: Add `--no-hidden` flag to skip hidden files (starting with `.`)
- **Output Formats**:
  - `--format csv` for machine-readable output
  - `--format json` for programmatic processing
- **Progress Indicators**: Add `--verbose` flag for real-time scanning progress
- **Parallel Processing**: Add parallel file hashing for large directories
- **Interactive Mode**: Interactive CLI to preview and delete duplicates
- **Exclude Patterns**: Add `--exclude` flag to skip certain file patterns (e.g., `*.tmp`)
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
