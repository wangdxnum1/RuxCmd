//! POSIX basic regular expression (BRE) translation to Rust regex syntax.
//!
//! Used by `expr` (`:` / `match`) and `sed` (addresses and the `s` command).
//! Back references (`\(a\)\1`) are not supported by the Rust regex engine and
//! surface as an error instead of silently mismatching.

/// Translate one BRE into an UNANCHORED Rust regex pattern. Callers decide
/// on anchoring (expr prepends `^`; sed searches anywhere).
pub fn to_regex(pattern: &str) -> Result<String, String> {
    to_regex_kind(pattern, false)
}

/// Translate one ERE into an UNANCHORED Rust regex pattern. ERE is close
/// enough to Rust regex syntax to pass through: unescaped `+ ? {m,n} | ( )`
/// keep their operator meaning, escaped ones stay literals.
pub fn to_regex_ere(pattern: &str) -> Result<String, String> {
    to_regex_kind(pattern, true)
}

/// Shared translation. Back references are unsupported in both dialects.
pub fn to_regex_kind(pattern: &str, extended: bool) -> Result<String, String> {
    if extended {
        return to_regex_ere_inner(pattern);
    }
    to_regex_bre(pattern)
}

fn to_regex_ere_inner(pattern: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                // Back references stay unsupported in ERE too.
                Some(d @ '1'..='9') => {
                    let _ = d;
                    return Err("back references are unsupported".to_string());
                }
                // Escaped ERE metacharacters stay escaped (literals).
                Some(other) => {
                    if "\\^$.[]|()?*+{}".contains(other) {
                        out.push('\\');
                    }
                    out.push(other);
                }
                None => out.push('\\'),
            },
            '^' => {
                if !out.is_empty() {
                    out.push('\\');
                }
                out.push('^');
            }
            '$' => {
                if chars.peek().is_none() {
                    out.push('$');
                } else {
                    out.push('\\');
                    out.push('$');
                }
            }
            '[' => {
                out.push('[');
                for rc in chars.by_ref() {
                    out.push(rc);
                    if rc == ']' {
                        break;
                    }
                }
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

fn to_regex_bre(pattern: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut chars = pattern.chars().peekable();
    let mut group_depth = 0usize;
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('(') => {
                    group_depth += 1;
                    out.push('(');
                }
                Some(')') => {
                    if group_depth == 0 {
                        return Err("unmatched closing group".to_string());
                    }
                    group_depth -= 1;
                    out.push(')');
                }
                Some('{') => out.push('{'),
                Some('}') => out.push('}'),
                // GNU BRE extensions: \+ is one-or-more, \? is
                // zero-or-one; Rust regex spells both unescaped.
                Some('+') => out.push('+'),
                Some('?') => out.push('?'),
                Some(d @ '1'..='9') => {
                    let _ = d;
                    return Err("back references are unsupported".to_string());
                }
                Some(other) => {
                    // An escaped metacharacter is literal; escape it for Rust.
                    if "\\.^$*+?()[]{}|".contains(other) {
                        out.push('\\');
                    }
                    out.push(other);
                }
                None => {
                    out.push('\\');
                }
            },
            '*' => out.push('*'),
            '.' => out.push('.'),
            '[' => {
                out.push('[');
                for rc in chars.by_ref() {
                    out.push(rc);
                    if rc == ']' {
                        break;
                    }
                }
            }
            '^' => {
                // A leading ^ anchors the expression in BRE; interior ones
                // are literal.
                if !out.is_empty() {
                    out.push('\\');
                }
                out.push('^');
            }
            '$' => {
                // BRE: literal anywhere except at the very end, where it is
                // an anchor that Rust regex also treats as end-of-string.
                if chars.peek().is_none() {
                    out.push('$');
                } else {
                    out.push('\\');
                    out.push('$');
                }
            }
            other => {
                if "|+?(){}".contains(other) {
                    out.push('\\');
                }
                out.push(other);
            }
        }
    }
    if group_depth != 0 {
        return Err("unmatched group".to_string());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{to_regex, to_regex_ere};

    #[test]
    fn groups_and_literals_translate() {
        assert_eq!(to_regex("a\\(.*\\)b").unwrap(), "a(.*)b");
        assert_eq!(to_regex("a.b").unwrap(), "a.b");
        assert_eq!(to_regex("a*b").unwrap(), "a*b");
        // GNU BRE extension: \+ is one-or-more, \? is zero-or-one.
        assert_eq!(to_regex("a\\+b").unwrap(), "a+b");
        assert_eq!(to_regex("a\\?b").unwrap(), "a?b");
        assert_eq!(to_regex("x\\{2,3\\}").unwrap(), "x{2,3}");
        assert_eq!(to_regex("a\\$b").unwrap(), "a\\$b");
        assert_eq!(to_regex("end$").unwrap(), "end$");
    }

    #[test]
    fn ere_passes_operators_through() {
        assert_eq!(to_regex_ere("a(b|c)+d").unwrap(), "a(b|c)+d");
        assert_eq!(to_regex_ere("x{2,3}").unwrap(), "x{2,3}");
        assert_eq!(to_regex_ere("colou?r").unwrap(), "colou?r");
        // Escaped metacharacters stay literals.
        assert_eq!(to_regex_ere("a\\+b").unwrap(), "a\\+b");
        assert_eq!(to_regex_ere("end$").unwrap(), "end$");
        // Back references stay unsupported in ERE too.
        assert!(to_regex_ere("(a)\\1").is_err());
    }

    #[test]
    fn back_references_error() {
        assert!(to_regex("\\(a\\)\\1").is_err());
    }

    #[test]
    fn unbalanced_groups_error() {
        assert!(to_regex("\\(a").is_err());
        assert!(to_regex("a\\)").is_err());
    }
}
