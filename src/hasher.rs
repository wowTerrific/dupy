use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::error::Result;

const CHUNK_SIZE: usize = 8192; // 8KB chunks

/// Compute a quick hash of the first 8KB of a file
pub fn quick_hash(path: &Path) -> Result<u64> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; CHUNK_SIZE];

    let bytes_read = reader.read(&mut buffer)?;

    // Simple hash: sum bytes with bit rotation for better distribution
    let mut hash: u64 = 0;
    for &byte in &buffer[..bytes_read] {
        hash = hash.rotate_left(5).wrapping_add(byte as u64);
    }

    Ok(hash)
}

/// Compare two files byte-by-byte to verify they are identical
pub fn files_identical(path1: &Path, path2: &Path) -> Result<bool> {
    let file1 = File::open(path1)?;
    let file2 = File::open(path2)?;

    let mut reader1 = BufReader::new(file1);
    let mut reader2 = BufReader::new(file2);

    let mut buffer1 = [0u8; CHUNK_SIZE];
    let mut buffer2 = [0u8; CHUNK_SIZE];

    loop {
        let bytes_read1 = reader1.read(&mut buffer1)?;
        let bytes_read2 = reader2.read(&mut buffer2)?;

        // If different amounts were read, files are different sizes
        if bytes_read1 != bytes_read2 {
            return Ok(false);
        }

        // If we've reached the end of both files, they're identical
        if bytes_read1 == 0 {
            return Ok(true);
        }

        // Compare the chunks
        if buffer1[..bytes_read1] != buffer2[..bytes_read2] {
            return Ok(false);
        }
    }
}
