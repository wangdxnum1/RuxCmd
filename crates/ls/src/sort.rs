use crate::cli::Args;
use crate::entry::{FileEntry, TimeField};

/// Sort entries based on the provided CLI arguments.
pub fn sort_entries(entries: &mut Vec<FileEntry>, args: &Args) {
    if args.unsorted {
        return; // Keep filesystem order
    }

    // Determine time field for sorting
    let time_field = if args.access_time {
        TimeField::Accessed
    } else if args.status_time {
        TimeField::Created
    } else {
        TimeField::Modified
    };

    // Primary sort
    if args.sort_size {
        entries.sort_by(|a, b| a.size().cmp(&b.size()));
    } else if args.sort_time {
        sort_by_time(entries, time_field);
    } else if args.sort_extension {
        entries.sort_by(|a, b| {
            ext(a.name.as_str())
                .cmp(&ext(b.name.as_str()))
                .then_with(|| compare_names(&a.name, &b.name))
        });
    } else if args.version_sort {
        entries.sort_by(|a, b| version_compare(&a.name, &b.name));
    } else {
        entries.sort_by(|a, b| compare_names(&a.name, &b.name));
    }

    // Group directories first if requested
    if args.group_dirs_first {
        entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| std::cmp::Ordering::Equal)
        });
    }

    // Reverse if requested
    if args.reverse {
        entries.reverse();
    }
}

fn sort_by_time(entries: &mut Vec<FileEntry>, time_field: TimeField) {
    entries.sort_by(|a, b| {
        let ta = get_time(a, time_field);
        let tb = get_time(b, time_field);
        tb.cmp(&ta) // newest first
            .then_with(|| compare_names(&a.name, &b.name))
    });
}

fn get_time(entry: &FileEntry, field: TimeField) -> std::time::SystemTime {
    match field {
        TimeField::Modified => entry.modified().unwrap_or(std::time::UNIX_EPOCH),
        TimeField::Accessed => entry.accessed().unwrap_or(std::time::UNIX_EPOCH),
        TimeField::Created => entry.created().unwrap_or(std::time::UNIX_EPOCH),
    }
}

/// Extract extension for sorting (empty string if no extension).
fn ext(name: &str) -> &str {
    name.rfind('.').map(|i| &name[i..]).unwrap_or("")
}

/// Natural version sort: split into numeric and non-numeric parts.
fn version_compare(a: &str, b: &str) -> std::cmp::Ordering {
    natural_compare(a, b)
}

/// Compare names case-insensitively (Windows-friendly), then case-sensitively.
fn compare_names(a: &str, b: &str) -> std::cmp::Ordering {
    let ci = a.to_lowercase().cmp(&b.to_lowercase());
    if ci != std::cmp::Ordering::Equal {
        ci
    } else {
        a.cmp(b)
    }
}

/// Simple natural sort — splits string into alternating text/number chunks.
fn natural_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let mut ia = ChunkIter::new(a);
    let mut ib = ChunkIter::new(b);
    loop {
        let ca = ia.next_str();
        let cb = ib.next_str();
        match (ca, cb) {
            (Some(sa), Some(sb)) => {
                let cmp = sa.cmp(sb);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
                // Continue to number chunks
            }
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (None, None) => return std::cmp::Ordering::Equal,
        }

        let na = ia.next_num();
        let nb = ib.next_num();
        match (na, nb) {
            (Some(va), Some(vb)) => {
                let cmp = va.cmp(&vb);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (None, None) => return std::cmp::Ordering::Equal,
        }
    }
}

struct ChunkIter<'a> {
    s: &'a str,
}

impl<'a> ChunkIter<'a> {
    fn new(s: &'a str) -> Self {
        ChunkIter { s }
    }

    fn next_str(&mut self) -> Option<&'a str> {
        let trimmed = self.s.trim_start();
        if trimmed.is_empty() {
            return None;
        }
        let end = trimmed
            .find(|c: char| c.is_ascii_digit())
            .unwrap_or(trimmed.len());
        if end == 0 {
            return None;
        }
        let (chunk, rest) = trimmed.split_at(end);
        self.s = rest;
        Some(chunk)
    }

    fn next_num(&mut self) -> Option<u64> {
        let trimmed = self.s.trim_start();
        if trimmed.is_empty() {
            return None;
        }
        let end = trimmed
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(trimmed.len());
        if end == 0 {
            return None;
        }
        let (chunk, rest) = trimmed.split_at(end);
        self.s = rest;
        chunk.parse::<u64>().ok()
    }
}
