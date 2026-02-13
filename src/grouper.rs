use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::hasher::{files_identical, quick_hash};
use crate::scanner::{walk_directory_names, FileInfo};

macro_rules! progress {
    ($verbose:expr, $($arg:tt)*) => {
        if $verbose { eprintln!($($arg)*); }
    };
}

pub struct NameDuplicateSet {
    pub name: OsString,
    pub files: Vec<PathBuf>,
}

pub fn find_name_duplicates(directory: &Path) -> Result<Vec<NameDuplicateSet>> {
    let entries = walk_directory_names(directory)?;
    let mut map: HashMap<OsString, Vec<PathBuf>> = HashMap::new();
    for (name, path) in entries {
        map.entry(name).or_default().push(path);
    }
    let mut result: Vec<NameDuplicateSet> = map
        .into_iter()
        .filter(|(_, paths)| paths.len() >= 2)
        .map(|(name, files)| NameDuplicateSet { name, files })
        .collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

#[derive(Debug)]
pub struct DuplicateSet {
    pub files: Vec<PathBuf>,
    pub size: u64,
}

pub fn find_duplicates(
    files: Vec<FileInfo>,
    min_size: u64,
    verbose: bool,
) -> Result<Vec<DuplicateSet>> {
    // Filter by min_size
    let files: Vec<FileInfo> = files.into_iter().filter(|f| f.size >= min_size).collect();

    // Stage 1: Group by file size, keep only groups with >= 2 files
    let size_groups: Vec<(u64, Vec<PathBuf>)> = group_by_size(files)
        .into_iter()
        .filter(|(_, paths)| paths.len() >= 2)
        .collect();

    progress!(
        verbose,
        "Checking {} size groups with potential duplicates...",
        size_groups.len()
    );

    // Stage 2: Quick hash — collect all hash groups with >= 2 files
    let mut hash_groups: Vec<(u64, Vec<PathBuf>)> = Vec::new();
    for (size, paths) in size_groups {
        for group in group_by_hash(&paths)? {
            if group.len() >= 2 {
                hash_groups.push((size, group));
            }
        }
    }

    progress!(verbose, "Verifying {} hash groups...", hash_groups.len());

    // Stage 3: Byte-by-byte verification
    let mut duplicate_sets = Vec::new();
    for (size, paths) in hash_groups {
        let verified_groups = verify_duplicates(paths)?;
        for group in verified_groups {
            if group.len() >= 2 {
                duplicate_sets.push(DuplicateSet { files: group, size });
            }
        }
    }

    // Sort descending by wasted space: size * (count - 1)
    duplicate_sets.sort_by(|a, b| {
        let wasted_a = a.size * (a.files.len() as u64 - 1);
        let wasted_b = b.size * (b.files.len() as u64 - 1);
        wasted_b.cmp(&wasted_a)
    });

    progress!(verbose, "Done.");

    Ok(duplicate_sets)
}

/// Stage 1: Group files by size
fn group_by_size(files: Vec<FileInfo>) -> HashMap<u64, Vec<PathBuf>> {
    let mut size_map: HashMap<u64, Vec<PathBuf>> = HashMap::new();

    for file in files {
        size_map.entry(file.size).or_default().push(file.path);
    }

    size_map
}

/// Stage 2: Group files by quick hash
fn group_by_hash(paths: &[PathBuf]) -> Result<Vec<Vec<PathBuf>>> {
    let mut hash_map: HashMap<u64, Vec<PathBuf>> = HashMap::new();

    for path in paths {
        // TODO - add logging to track which files we can't hash
        // Skip files we can't hash (permission issues, etc.)
        if let Ok(hash) = quick_hash(path) {
            hash_map.entry(hash).or_default().push(path.clone());
        }
    }

    Ok(hash_map.into_values().collect())
}

/// Stage 3: Verify duplicates with byte-by-byte comparison
fn verify_duplicates(paths: Vec<PathBuf>) -> Result<Vec<Vec<PathBuf>>> {
    let mut verified_groups = Vec::new();
    let mut remaining = paths;

    while !remaining.is_empty() {
        let reference = remaining.remove(0);
        let mut group = vec![reference.clone()];

        let mut i = 0;
        while i < remaining.len() {
            // Skip files we can't compare (permission issues, etc.)
            if let Ok(identical) = files_identical(&reference, &remaining[i]) {
                if identical {
                    group.push(remaining.remove(i));
                    continue;
                }
            }
            i += 1;
        }

        if group.len() >= 2 {
            verified_groups.push(group);
        }
    }

    Ok(verified_groups)
}
