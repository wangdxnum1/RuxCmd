use colored::*;

use crate::entry::FileEntry;

/// Style a filename according to its type (mimicking GNU ls colors).
pub fn colorize_name(entry: &FileEntry, name: &str) -> String {
    if entry.is_symlink {
        name.cyan().to_string()
    } else if entry.is_dir {
        name.blue().bold().to_string()
    } else if is_executable(&entry.name) {
        name.green().bold().to_string()
    } else {
        // Default color
        name.normal().to_string()
    }
}

/// Return the indicator character for classify (-F) mode.
pub fn classify_indicator(entry: &FileEntry) -> &'static str {
    if entry.is_symlink {
        "@"
    } else if entry.is_dir {
        "/"
    } else if is_executable(&entry.name) {
        "*"
    } else {
        ""
    }
}

fn is_executable(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".exe")
        || lower.ends_with(".com")
        || lower.ends_with(".bat")
        || lower.ends_with(".cmd")
        || lower.ends_with(".ps1")
        || lower.ends_with(".py")
        || lower.ends_with(".pl")
        || lower.ends_with(".vbs")
        || lower.ends_with(".js")
        || lower.ends_with(".msi")
}
