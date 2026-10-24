//! Windows verbatim-path normalization for display.
//!
//! `fs::canonicalize` returns `\\?\C:\...` (or `\\?\UNC\server\share\...`)
//! paths that scripts cannot reuse directly. These helpers strip the
//! verbatim prefixes and normalize separators to `/`, matching the output
//! convention of realpath/dirname/which/readlink.

use std::path::Path;

/// Display form of a path: `\\?\UNC\server\share\x` becomes
/// `//server/share/x`, `\\?\C:\x` becomes `C:/x`, backslashes become `/`.
pub fn display(path: &Path) -> String {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        // Render UNC roots the way the path tools print them.
        return format!("//{}", rest).replace('\\', "/");
    }
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_verbatim_prefix_is_stripped() {
        assert_eq!(display(Path::new(r"\\?\C:\tmp\a")), "C:/tmp/a");
    }

    #[test]
    fn unc_verbatim_maps_to_double_slash() {
        assert_eq!(
            display(Path::new(r"\\?\UNC\server\share\a")),
            "//server/share/a"
        );
    }

    #[test]
    fn plain_paths_just_swap_separators() {
        assert_eq!(display(Path::new(r"C:\tmp\a")), "C:/tmp/a");
        assert_eq!(display(Path::new("a/b")), "a/b");
    }
}
