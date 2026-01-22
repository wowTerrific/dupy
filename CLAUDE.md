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

This design prioritizes performance by filtering aggressively at each stage before doing expensive operations.

### Module Responsibilities

- **error.rs**: Custom error types with automatic conversion from `std::io::Error`
- **scanner.rs**: Recursive directory traversal that gracefully skips inaccessible files/symlinks
- **hasher.rs**:
  - `quick_hash()`: Computes hash of first 8KB using bit rotation
  - `files_identical()`: Chunk-by-chunk comparison (8KB chunks)
- **grouper.rs**: Orchestrates the 3-stage pipeline, managing `HashMap` groupings at each stage
- **formatter.rs**: Human-readable output with byte formatting (B/KB/MB/GB)
- **main.rs**: CLI argument parsing and error handling (exits with code 1 on error)

### Key Constraints

- **No external dependencies**: Keep `Cargo.toml` dependencies section empty
- **Memory efficiency**: Never load entire files into memory - always use 8KB buffered chunks
- **Error resilience**: Non-fatal errors (permission denied, corrupted files) should log and continue, not crash

## Common Commands

```bash
# Build release binary
cargo build --release

# Run on a directory
./target/release/dupy <directory>

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
```

## Making Changes

### Adding New Features

When adding CLI flags or new functionality:

1. **Update `main.rs`** for argument parsing
2. **Modify the pipeline** in `grouper.rs` if changing duplicate detection logic
3. **Update `formatter.rs`** if changing output format
4. **Keep it zero-dependency** - no external crates allowed

### Performance Considerations

- The 8KB chunk size (`CHUNK_SIZE` in `hasher.rs`) balances I/O efficiency and memory usage
- HashMap groupings use `.or_default()` for cleaner code (clippy preference)
- Early bailouts prevent unnecessary work (skip size groups with 1 file)

### Error Handling Pattern

Use the custom `Result<T>` type alias from `error.rs` throughout. The `?` operator automatically converts `std::io::Error` via the `From` trait implementation. Fatal errors return early; non-fatal errors (file access issues) skip the file and continue.
