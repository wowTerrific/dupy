use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
}

pub fn walk_directory(path: &Path) -> Result<Vec<FileInfo>> {
    let mut files = Vec::new();
    walk_directory_recursive(path, &mut files)?;
    Ok(files)
}

fn walk_directory_recursive(path: &Path, files: &mut Vec<FileInfo>) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return Ok(()), // Skip inaccessible files
    };

    if metadata.is_file() {
        files.push(FileInfo {
            path: path.to_path_buf(),
            size: metadata.len(),
        });
    } else if metadata.is_dir() {
        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => return Ok(()), // Skip inaccessible directories
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // Skip problematic entries
            };

            let entry_path = entry.path();
            walk_directory_recursive(&entry_path, files)?;
        }
    }
    // Skip symlinks and other special files

    Ok(())
}
