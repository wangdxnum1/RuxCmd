/// Windows file attribute constants
mod attr {
    pub const READONLY: u32 = 0x01;
    pub const HIDDEN: u32 = 0x02;
}

/// Convert Windows file attributes to a Unix-style permission string (10 chars).
///
/// The first character indicates the file type:
///   `-` regular file, `d` directory, `l` symlink
///
/// The remaining 9 characters are a simplified rwx triplet.
pub fn format_permissions(file_attributes: u32, is_dir: bool, is_symlink: bool, name: &str) -> String {
    let mut result = String::with_capacity(10);

    // File type character
    if is_symlink {
        result.push('l');
    } else if is_dir {
        result.push('d');
    } else {
        result.push('-');
    }

    let is_readonly = file_attributes & attr::READONLY != 0;
    let is_hidden = file_attributes & attr::HIDDEN != 0;

    // Check if the file is "executable" by extension
    let is_exe = is_executable(name);

    // Owner permissions
    if is_dir {
        result.push('r');
        result.push('w');
        result.push('x');
    } else if is_readonly {
        result.push('r');
        result.push('-');
        result.push(if is_exe { 'x' } else { '-' });
    } else {
        result.push('r');
        result.push('w');
        result.push(if is_exe { 'x' } else { '-' });
    }

    // Group permissions (simplified: same as owner but no write)
    if is_dir {
        result.push('r');
        result.push('-');
        result.push('x');
    } else if is_readonly {
        result.push('r');
        result.push('-');
        result.push(if is_exe { 'x' } else { '-' });
    } else {
        result.push('r');
        result.push('-');
        result.push(if is_exe { 'x' } else { '-' });
    }

    // Other permissions (read-only always)
    if is_dir {
        result.push('r');
        result.push('-');
        result.push('x');
    } else {
        result.push('r');
        result.push('-');
        result.push(if is_exe { 'x' } else { '-' });
    }

    // Show hidden attribute as a subtle marker — replace last char
    if is_hidden {
        result.pop();
        result.push('h');
    }

    result
}

/// Check if a filename has an extension considered executable on Windows.
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

/// Get the owner display string (uses username from env on Windows).
pub fn get_owner_name() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Get the group display string (uses USERDOMAIN on Windows).
pub fn get_group_name() -> String {
    std::env::var("USERDOMAIN")
        .unwrap_or_else(|_| "None".to_string())
}
