use chrono::{DateTime, Local, TimeDelta};

use crate::cli::Args;
use crate::color;
use crate::entry::{FileEntry, TimeField};
use crate::perm;

/// Maximum width for terminal columns.
const DEFAULT_TERMINAL_WIDTH: usize = 80;

/// Print all entries according to the CLI args.
pub fn print_entries(entries: &[FileEntry], args: &Args) {
    if entries.is_empty() {
        return;
    }

    // Filter entries for display
    let filtered: Vec<&FileEntry> = entries
        .iter()
        .filter(|e| args.show_hidden() || !e.name.starts_with('.'))
        .filter(|e| !(args.ignore_backups && e.name.ends_with('~')))
        .collect();

    if filtered.is_empty() {
        return;
    }

    if args.long {
        print_long(&filtered, args);
    } else if args.one_per_line {
        print_one_per_line(&filtered, args);
    } else if args.comma {
        print_comma(&filtered, args);
    } else if args.across {
        print_across(&filtered, args);
    } else {
        print_columns(&filtered, args);
    }

    // Handle recursive listing
    if args.recursive {
        let dirs: Vec<&FileEntry> = filtered.iter().filter(|e| e.is_dir).copied().collect();
        for dir in dirs {
            println!();
            if let Ok(sub_entries) = crate::entry::collect_entries(&dir.path, args) {
                if !sub_entries.is_empty() {
                    println!("{}:", dir.path.display());
                    let total_blocks: u64 = sub_entries
                        .iter()
                        .filter(|e| args.show_hidden() || !e.name.starts_with('.'))
                        .map(|e| e.block_count() * 512 / 1024)
                        .sum();
                    if args.long {
                        println!("total {}", total_blocks);
                    }
                    print_entries(&sub_entries, args);
                }
            }
        }
    }
}

/// Column-based output (like -C, the default).
fn print_columns(entries: &[&FileEntry], args: &Args) {
    let color_enabled = args.color_enabled();
    let term_width = args.width.unwrap_or_else(|| {
        term_size().unwrap_or(DEFAULT_TERMINAL_WIDTH)
    });

    // Determine prefix width for -i and -s
    let prefix_width = prefix_len(entries, args);

    // Calculate max name width
    let max_width = entries
        .iter()
        .map(|e| display_name(e, args))
        .max()
        .unwrap_or(0);
    let col_width = max_width + 1 + prefix_width; // +1 for spacing

    if col_width >= term_width || max_width == 0 {
        // Fall back to single column if names are too wide
        for entry in entries {
            let prefix = format_prefix(entry, args, prefix_width);
            let name = format_name(entry, args, color_enabled);
            println!("{}{}", prefix, name);
        }
        return;
    }

    let num_cols = (term_width / col_width).max(1);
    let num_rows = (entries.len() + num_cols - 1) / num_cols;

    // Standard column layout (going down then across)
    for row in 0..num_rows {
        let mut line = String::new();
        for col in 0..num_cols {
            let idx = col * num_rows + row;
            if idx < entries.len() {
                let entry = entries[idx];
                let prefix = format_prefix(entry, args, prefix_width);
                let name = format_name(entry, args, color_enabled);
                line.push_str(&prefix);
                line.push_str(&name);
                // Pad to column width
                let display_len = display_name(entry, args);
                let padding = col_width.saturating_sub(display_len + prefix_width);
                for _ in 0..padding {
                    line.push(' ');
                }
            }
        }
        println!("{}", line.trim_end());
    }
}

/// Calculate the width needed for the -i / -s prefix.
fn prefix_len(entries: &[&FileEntry], args: &Args) -> usize {
    let mut w = 0usize;
    if args.inode {
        w += 9; // max inode width approximation
    }
    if args.size {
        if args.inode {
            w += 1; // space
        }
        let max_blocks = entries
            .iter()
            .map(|e| e.block_count())
            .max()
            .unwrap_or(0);
        w += format!("{}", max_blocks).len();
    }
    w
}

/// Format the -i / -s prefix for an entry.
fn format_prefix(entry: &FileEntry, args: &Args, width: usize) -> String {
    let mut parts = Vec::new();
    if args.inode {
        if let Some(ino) = entry.file_index() {
            parts.push(format!("{}", ino));
        } else {
            parts.push("?".to_string());
        }
    }
    if args.size {
        parts.push(format!("{}", entry.block_count()));
    }
    let joined = parts.join(" ");
    // Left-pad to width
    format!("{:>width$} ", joined, width = width)
}

/// Across output (-x): list entries left-to-right, top-to-bottom.
fn print_across(entries: &[&FileEntry], args: &Args) {
    let color_enabled = args.color_enabled();
    let term_width = args.width.unwrap_or_else(|| {
        term_size().unwrap_or(DEFAULT_TERMINAL_WIDTH)
    });

    let prefix_width = prefix_len(entries, args);

    // Calculate max name width
    let max_width = entries
        .iter()
        .map(|e| display_name(e, args))
        .max()
        .unwrap_or(0);
    let col_width = max_width + 1 + prefix_width;

    if col_width >= term_width || max_width == 0 {
        // Fall back to single column
        for entry in entries {
            let prefix = format_prefix(entry, args, prefix_width);
            let name = format_name(entry, args, color_enabled);
            println!("{}{}", prefix, name);
        }
        return;
    }

    let num_cols = (term_width / col_width).max(1);

    // Across: iterate left-to-right, top-to-bottom
    for (i, entry) in entries.iter().enumerate() {
        let prefix = format_prefix(entry, args, prefix_width);
        let name = format_name(entry, args, color_enabled);
        let entry_str = format!("{}{}", prefix, name);
        print!("{}", entry_str);

        if (i + 1) % num_cols == 0 || i == entries.len() - 1 {
            println!();
        } else {
            let display_len = prefix_width + display_name(entry, args);
            let padding = col_width.saturating_sub(display_len);
            for _ in 0..padding {
                print!(" ");
            }
        }
    }
}

/// Single-column output (-1).
fn print_one_per_line(entries: &[&FileEntry], args: &Args) {
    let color_enabled = args.color_enabled();
    let prefix_width = prefix_len(entries, args);
    for entry in entries {
        let prefix = format_prefix(entry, args, prefix_width);
        let name = format_name(entry, args, color_enabled);
        println!("{}{}", prefix, name);
    }
}

/// Comma-separated output (-m).
fn print_comma(entries: &[&FileEntry], args: &Args) {
    let color_enabled = args.color_enabled();
    let term_width = args.width.unwrap_or_else(|| {
        term_size().unwrap_or(DEFAULT_TERMINAL_WIDTH)
    });
    let prefix_width = if args.inode || args.size {
        prefix_len(entries, args) + 1
    } else {
        0
    };

    let mut line = String::new();
    for (i, entry) in entries.iter().enumerate() {
        let prefix = if prefix_width > 0 {
            format_prefix(entry, args, prefix_width - 1)
        } else {
            String::new()
        };
        let name = format_name(entry, args, color_enabled);
        let separator = if i < entries.len() - 1 { ", " } else { "" };
        let segment = format!("{}{}{}", prefix, name, separator);

        if line.len() + segment.len() > term_width && !line.is_empty() {
            println!("{}", line.trim_end());
            line = segment;
        } else {
            line.push_str(&segment);
        }
    }
    if !line.is_empty() {
        println!("{}", line.trim_end());
    }
}

/// Long format output (-l).
fn print_long(entries: &[&FileEntry], args: &Args) {
    let color_enabled = args.color_enabled();

    // Pre-compute all fields for alignment
    struct LongFields {
        permissions: String,
        nlink: String,
        owner: String,
        group: String,
        size: String,
        time: String,
        name: String,
    }

    let time_field = if args.access_time {
        TimeField::Accessed
    } else if args.status_time {
        TimeField::Created
    } else {
        TimeField::Modified
    };

    let fields: Vec<LongFields> = entries
        .iter()
        .map(|entry| {
            let permissions = perm::format_permissions(
                entry.file_attributes(),
                entry.is_dir,
                entry.is_symlink,
                &entry.name,
            );
            let nlink = entry.nlink().to_string();
            let owner = if args.show_owner() {
                if args.numeric_uid_gid {
                    "0".to_string() // Simplified: no real UID on Windows
                } else {
                    perm::get_owner_name()
                }
            } else {
                String::new()
            };
            let group = if args.show_group() {
                if args.numeric_uid_gid {
                    "0".to_string()
                } else {
                    perm::get_group_name()
                }
            } else {
                String::new()
            };
            let size = format_size(entry, args);
            let time = format_time(entry, time_field, args);
            let name = format_name(entry, args, color_enabled);

            LongFields {
                permissions,
                nlink,
                owner,
                group,
                size,
                time,
                name,
            }
        })
        .collect();

    // Calculate column widths
    let max_nlink = fields.iter().map(|f| f.nlink.len()).max().unwrap_or(1);
    let max_owner = fields.iter().map(|f| f.owner.len()).max().unwrap_or(0);
    let max_group = fields.iter().map(|f| f.group.len()).max().unwrap_or(0);
    let max_size = fields.iter().map(|f| f.size.len()).max().unwrap_or(1);

    for f in &fields {
        let mut line = String::new();

        // Inode (-i)
        if args.inode {
            if let Some(idx) = entries
                .iter()
                .position(|e| e.name == extract_name_from_longfield(&f.name))
            {
                if let Some(ino) = entries[idx].file_index() {
                    line.push_str(&format!("{} ", ino));
                } else {
                    line.push_str("? ");
                }
            }
        }

        // Block count (-s)
        if args.size {
            if let Some(idx) = entries
                .iter()
                .position(|e| e.name == extract_name_from_longfield(&f.name))
            {
                let blocks = entries[idx].block_count();
                line.push_str(&format!("{} ", blocks));
            }
        }

        line.push_str(&f.permissions);
        line.push_str("  ");
        line.push_str(&pad_right(&f.nlink, max_nlink));
        line.push(' ');

        if args.show_owner() {
            line.push_str(&pad_right(&f.owner, max_owner));
            line.push(' ');
        }
        if args.show_group() {
            line.push_str(&pad_right(&f.group, max_group));
            line.push(' ');
        }

        line.push_str(&pad_right(&f.size, max_size));
        line.push(' ');
        line.push_str(&f.time);
        line.push(' ');
        line.push_str(&f.name);

        // Add symlink target if applicable
        let name_plain = extract_name_from_longfield(&f.name);
        if let Some(entry) = entries.iter().find(|e| e.name == name_plain) {
            if let Some(ref target) = entry.symlink_target {
                line.push_str(" -> ");
                line.push_str(target);
            }
        }

        println!("{}", line);
    }
}

fn extract_name_from_longfield(name: &str) -> &str {
    // Strip ANSI color codes by finding the plain text portion
    // For simplicity, we just use the name as-is since colored text
    // might include escape sequences. We match by entry name directly.
    // This function is kept for clarity.
    name
}

/// Format the entry name with optional color, classify indicator, and quoting.
fn format_name(entry: &FileEntry, args: &Args, color_enabled: bool) -> String {
    let name = &entry.name;

    let display = if args.quote_name {
        format!("\"{}\"", name)
    } else {
        name.clone()
    };

    let indicator = if args.classify {
        color::classify_indicator(entry)
    } else if args.indicator_slash && entry.is_dir {
        "/"
    } else {
        ""
    };

    let colored = if color_enabled {
        color::colorize_name(entry, &display)
    } else {
        display
    };

    format!("{}{}", colored, indicator)
}

/// Get the display name length (without color codes) for column alignment.
fn display_name(entry: &FileEntry, args: &Args) -> usize {
    let mut len = entry.name.len();
    if args.quote_name {
        len += 2; // quotes
    }
    if args.classify {
        len += color::classify_indicator(entry).len();
    } else if args.indicator_slash && entry.is_dir {
        len += 1;
    }
    len
}

/// Format file size according to -h, --block-size, etc.
fn format_size(entry: &FileEntry, args: &Args) -> String {
    let bytes = entry.size();
    if args.human_readable {
        human_readable_size(bytes)
    } else if let Some(ref bs) = args.block_size {
        format_size_with_block_size(bytes, bs)
    } else if args.kibibytes {
        format!("{}", bytes / 1024)
    } else {
        format!("{}", bytes)
    }
}

fn human_readable_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["", "K", "M", "G", "T", "P", "E"];
    if bytes == 0 {
        return "0".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{}", bytes)
    } else if size >= 10.0 {
        format!("{:.0}{}", size, UNITS[unit_idx])
    } else {
        format!("{:.1}{}", size, UNITS[unit_idx])
    }
}

fn format_size_with_block_size(bytes: u64, block_size: &str) -> String {
    let factor = match block_size.to_uppercase().as_str() {
        "K" | "KIB" => 1024u64,
        "M" | "MIB" => 1024u64 * 1024,
        "G" | "GIB" => 1024u64 * 1024 * 1024,
        "T" | "TIB" => 1024u64 * 1024 * 1024 * 1024,
        "KB" => 1000,
        "MB" => 1000 * 1000,
        "GB" => 1000 * 1000 * 1000,
        "TB" => 1000 * 1000 * 1000 * 1000,
        _ => 1,
    };
    let scaled = bytes / factor;
    format!("{}", scaled)
}

/// Format timestamp for -l output according to --time-style.
fn format_time(entry: &FileEntry, field: TimeField, args: &Args) -> String {
    let sys_time = match field {
        TimeField::Modified => entry.modified(),
        TimeField::Accessed => entry.accessed(),
        TimeField::Created => entry.created(),
    };

    let sys_time = match sys_time {
        Some(t) => t,
        None => return "---------- --:--".to_string(),
    };

    let dt: DateTime<Local> = DateTime::from(sys_time);
    let now = Local::now();
    let six_months_ago = now - TimeDelta::days(180);

    let time_style = args.time_style.as_deref().unwrap_or("locale");

    match time_style {
        "full-iso" => dt.format("%Y-%m-%d %H:%M:%S.%f %z").to_string(),
        "long-iso" => dt.format("%Y-%m-%d %H:%M").to_string(),
        "iso" => dt.format("%m-%d %H:%M").to_string(),
        "locale" => {
            // GNU ls default: if date is within last 6 months, show month day hh:mm
            // Otherwise show month day  year
            if dt > six_months_ago && dt < now {
                dt.format("%b %e %H:%M").to_string()
            } else {
                dt.format("%b %e  %Y").to_string()
            }
        }
        // Custom format starting with +
        other if other.starts_with('+') => {
            let fmt = &other[1..];
            dt.format(fmt).to_string()
        }
        _ => dt.format("%b %e %H:%M").to_string(),
    }
}

fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        let mut result = s.to_string();
        for _ in 0..(width - s.len()) {
            result.push(' ');
        }
        result
    }
}

/// Try to get terminal width on Windows.
fn term_size() -> Option<usize> {
    #[cfg(windows)]
    {
        const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5u32;
        type DWORD = u32;
        type BOOL = i32;
        type HANDLE = isize;

        #[repr(C)]
        #[allow(non_snake_case)]
        struct CONSOLE_SCREEN_BUFFER_INFO {
            dwSize: Coord,
            dwCursorPosition: Coord,
            wAttributes: u16,
            srWindow: SmallRect,
            dwMaximumWindowSize: Coord,
        }

        #[repr(C)]
        #[allow(non_snake_case)]
        struct Coord {
            X: i16,
            Y: i16,
        }

        #[repr(C)]
        #[allow(non_snake_case)]
        struct SmallRect {
            Left: i16,
            Top: i16,
            Right: i16,
            Bottom: i16,
        }

        unsafe extern "system" {
            fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
            fn GetConsoleScreenBufferInfo(
                hConsoleOutput: HANDLE,
                lpConsoleScreenBufferInfo: *mut CONSOLE_SCREEN_BUFFER_INFO,
            ) -> BOOL;
        }

        unsafe {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if handle == -1isize {
                return None;
            }
            let mut info: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
            if GetConsoleScreenBufferInfo(handle, &mut info) != 0 {
                Some((info.srWindow.Right - info.srWindow.Left + 1) as usize)
            } else {
                None
            }
        }
    }
    #[cfg(not(windows))]
    {
        None
    }
}
