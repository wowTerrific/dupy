use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::Result;
use crate::hasher::{files_identical, quick_hash};
use crate::scanner::FileInfo;

#[derive(Debug)]
pub struct DuplicateSet {
    pub files: Vec<PathBuf>,
    pub size: u64,
}

pub fn find_duplicates(files: Vec<FileInfo>) -> Result<Vec<DuplicateSet>> {
    // Stage 1: Group by file size
    let size_groups = group_by_size(files);

    let mut duplicate_sets = Vec::new();

    // Stage 2 & 3: For each size group, use quick hash and verify
    for (size, paths) in size_groups {
        // Skip groups with only one file
        if paths.len() < 2 {
            continue;
        }

        // Stage 2: Group by quick hash
        let hash_groups = group_by_hash(&paths)?;

        // Stage 3: Verify each hash group with byte-by-byte comparison
        for hash_paths in hash_groups {
            if hash_paths.len() < 2 {
                continue;
            }

            let verified_groups = verify_duplicates(hash_paths)?;

            for group in verified_groups {
                if group.len() >= 2 {
                    duplicate_sets.push(DuplicateSet { files: group, size });
                }
            }
        }
    }

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
