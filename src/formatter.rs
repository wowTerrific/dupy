use crate::grouper::{DuplicateSet, NameDuplicateSet};

pub fn format_table(duplicate_sets: &[DuplicateSet]) {
    if duplicate_sets.is_empty() {
        println!("No duplicate files found.");
        return;
    }

    println!("Duplicate Files Report");
    println!("======================\n");

    let mut total_files = 0;
    let mut total_wasted: u64 = 0;

    for (idx, set) in duplicate_sets.iter().enumerate() {
        let wasted = set.size * (set.files.len() as u64 - 1);
        total_files += set.files.len();
        total_wasted += wasted;

        println!(
            "Group {} ({} files, {} each, {} wasted):",
            idx + 1,
            set.files.len(),
            format_bytes(set.size),
            format_bytes(wasted)
        );

        for file in &set.files {
            println!("  {}", file.display());
        }
        println!();
    }

    println!("Summary:");
    println!("--------");
    println!("Total duplicate groups: {}", duplicate_sets.len());
    println!(
        "Total duplicate files: {} (of which {} are redundant copies)",
        total_files,
        total_files - duplicate_sets.len()
    );
    println!("Total wasted space: {}", format_bytes(total_wasted));
}

pub fn format_name_table(groups: &[NameDuplicateSet]) {
    if groups.is_empty() {
        println!("No files with duplicate names found.");
        return;
    }
    println!("Duplicate Names Report");
    println!("======================\n");
    let mut total_files = 0;
    for (idx, group) in groups.iter().enumerate() {
        total_files += group.files.len();
        println!(
            "Group {} - \"{}\" ({} files):",
            idx + 1,
            group.name.to_string_lossy(),
            group.files.len()
        );
        for file in &group.files {
            println!("  {}", file.display());
        }
        println!();
    }
    println!("Summary:");
    println!("--------");
    println!("Total groups with shared names: {}", groups.len());
    println!("Total files involved: {}", total_files);
}

pub fn format_csv(sets: &[DuplicateSet]) {
    println!("group_id,size_bytes,size_human,wasted_bytes,wasted_human,file_path");
    for (idx, set) in sets.iter().enumerate() {
        let wasted = set.size * (set.files.len() as u64 - 1);
        let size_h = format_bytes(set.size);
        let wasted_h = format_bytes(wasted);
        for file in &set.files {
            println!(
                "{},{},{},{},{},{}",
                idx + 1,
                set.size,
                csv_escape(&size_h),
                wasted,
                csv_escape(&wasted_h),
                csv_escape(&file.display().to_string()),
            );
        }
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
