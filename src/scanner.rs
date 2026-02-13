use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
}

const DEFAULT_EXCLUDES: &[&str] = &[
    "Thumbs.db",
    ".DS_Store",
    "desktop.ini",
    "~$*",
    "*.tmp",
    "*.lnk",
];

pub fn walk_directory(
    path: &Path,
    user_excludes: &[String],
    include_junk: bool,
) -> Result<Vec<FileInfo>> {
    let mut active: Vec<&str> = user_excludes.iter().map(|s| s.as_str()).collect();
    if !include_junk {
        active.extend_from_slice(DEFAULT_EXCLUDES);
    }
    let mut files = Vec::new();
    walk_directory_recursive(path, &mut files, &active)?;
    Ok(files)
}

fn walk_directory_recursive(
    path: &Path,
    files: &mut Vec<FileInfo>,
    excludes: &[&str],
) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        // TODO- add logging to track which files we can't access
        Err(_) => return Ok(()), // Skip inaccessible files
    };

    if metadata.is_file() {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if excludes
                .iter()
                .any(|pat| crate::glob::glob_match(pat, name))
            {
                return Ok(());
            }
        }
        files.push(FileInfo {
            path: path.to_path_buf(),
            size: metadata.len(),
        });
    } else if metadata.is_dir() {
        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => {
                // TODO- add logging to track which directories we can't access
                return Ok(());
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // Skip problematic entries
            };

            let entry_path = entry.path();
            walk_directory_recursive(&entry_path, files, excludes)?;
        }
    }
    // Skip symlinks and other special files

    Ok(())
}

pub fn walk_directory_names(path: &Path) -> Result<Vec<(OsString, PathBuf)>> {
    let mut entries = Vec::new();
    walk_names_recursive(path, &mut entries)?;
    Ok(entries)
}

fn walk_names_recursive(path: &Path, entries: &mut Vec<(OsString, PathBuf)>) -> Result<()> {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };
    if metadata.is_file() {
        if let Some(name) = path.file_name() {
            entries.push((name.to_os_string(), path.to_path_buf()));
        }
    } else if metadata.is_dir() {
        let dir_entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };
        for entry in dir_entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            walk_names_recursive(&entry.path(), entries)?;
        }
    }
    // symlinks and special files skipped — same policy as existing walk
    Ok(())
}
