//! POSIX basic regular expression (BRE) translation to Rust regex syntax.
//!
//! Used by `expr` (`:` / `match`) and `sed` (addresses and the `s` command).
//! The [`to_regex`] / [`to_regex_ere`] paths emit patterns for the `regex`
//! crate, which has no back reference support, so there `\1`-`\9` surface as
//! an error instead of silently mismatching. The [`to_regex_fancy`] /
//! [`to_regex_ere_fancy`] paths instead emit patterns for the `fancy-regex`
//! crate (used by `expr` and `sed`) and pass back references through, keeping
//! GNU's validity rule: `\N` is accepted only when the Nth group is already
//! closed at that point (in BRE the Nth `\(...\)`, in ERE the Nth `(...)` —
//! GNU supports back references in ERE as an extension).

/// Translate one BRE into an UNANCHORED Rust regex pattern. Callers decide
/// on anchoring (expr prepends `^`; sed searches anywhere).
pub fn to_regex(pattern: &str) -> Result<String, String> {
    to_regex_bre(pattern, false)
}

/// Translate one BRE into an UNANCHORED `fancy-regex` pattern. Identical to
/// [`to_regex`] except that back references `\1`-`\9` referring to already
/// closed groups are passed through (fancy-regex implements them); anything
/// else is GNU's "Invalid back reference".
pub fn to_regex_fancy(pattern: &str) -> Result<String, String> {
    to_regex_bre(pattern, true)
}

/// Translate one ERE into an UNANCHORED Rust regex pattern. ERE is close
/// enough to Rust regex syntax to pass through: unescaped `+ ? {m,n} | ( )`
/// keep their operator meaning, escaped ones stay literals.
pub fn to_regex_ere(pattern: &str) -> Result<String, String> {
    to_regex_ere_inner(pattern, false)
}

/// Translate one ERE into an UNANCHORED `fancy-regex` pattern. Identical to
/// [`to_regex_ere`] except that GNU's back reference extension `\1`-`\9`
/// (which `sed -E` honors too) is passed through when the referenced group
/// is already closed.
pub fn to_regex_ere_fancy(pattern: &str) -> Result<String, String> {
    to_regex_ere_inner(pattern, true)
}

/// Shared translation into strict regex-crate patterns. Back references are
/// unsupported in both dialects.
pub fn to_regex_kind(pattern: &str, extended: bool) -> Result<String, String> {
    if extended {
        return to_regex_ere_inner(pattern, false);
    }
    to_regex_bre(pattern, false)
}

fn to_regex_ere_inner(pattern: &str, allow_backrefs: bool) -> Result<String, String> {
    let mut out = String::new();
    let mut chars = pattern.chars().peekable();
    // Back reference validity needs the group state: groups close in LIFO
    // order, so at any point the closed groups are exactly 1..=closed, and
    // `\N` is valid iff N <= closed. Escaped parens and parens inside bracket
    // expressions are literals and never touch the counters.
    let mut opened = 0usize;
    let mut closed = 0usize;
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some(d @ '1'..='9') => {
                    if !allow_backrefs {
                        return Err("back references are unsupported".to_string());
                    }
                    let n = d as usize - '0' as usize;
                    if n > closed {
                        return Err("Invalid back reference".to_string());
                    }
                    // fancy-regex understands `\N` verbatim.
                    out.push('\\');
                    out.push(d);
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
            '(' => {
                opened += 1;
                out.push('(');
            }
            ')' => {
                if closed == opened {
                    // GNU rejects an unmatched closing paren up front.
                    return Err("Unmatched ) or \\)".to_string());
                }
                closed += 1;
                out.push(')');
            }
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

fn to_regex_bre(pattern: &str, allow_backrefs: bool) -> Result<String, String> {
    let mut out = String::new();
    let mut chars = pattern.chars().peekable();
    // Groups are numbered by opening order; each entry records whether that
    // group's closing `\)` has been seen. `open` holds the numbers of the
    // groups still unclosed, innermost last.
    let mut groups: Vec<bool> = Vec::new();
    let mut open: Vec<usize> = Vec::new();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('(') => {
                    groups.push(false);
                    open.push(groups.len());
                    out.push('(');
                }
                Some(')') => {
                    let number = open
                        .pop()
                        .ok_or_else(|| "unmatched closing group".to_string())?;
                    groups[number - 1] = true;
                    out.push(')');
                }
                Some('{') => out.push('{'),
                Some('}') => out.push('}'),
                // GNU BRE extensions: \+ is one-or-more, \? is
                // zero-or-one; Rust regex spells both unescaped.
                Some('+') => out.push('+'),
                Some('?') => out.push('?'),
                Some(d @ '1'..='9') => {
                    if !allow_backrefs {
                        return Err("back references are unsupported".to_string());
                    }
                    // GNU validity rule: the Nth group must exist and be
                    // closed before the reference (so forward references and
                    // references into an enclosing group are errors).
                    let n = d as usize - '0' as usize;
                    if n > groups.len() || !groups[n - 1] {
                        return Err("Invalid back reference".to_string());
                    }
                    // fancy-regex understands `\N` verbatim.
                    out.push('\\');
                    out.push(d);
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
    if !open.is_empty() {
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
        // The strict ERE path keeps rejecting back references.
        assert!(to_regex_ere("(a)\\1").is_err());
    }

    #[test]
    fn back_references_error() {
        // The strict regex-crate path keeps rejecting back references.
        assert!(to_regex("\\(a\\)\\1").is_err());
    }

    #[test]
    fn ere_back_references_translate_for_fancy() {
        use super::to_regex_ere_fancy;

        assert_eq!(to_regex_ere_fancy("(a)\\1").unwrap(), "(a)\\1");
        // Groups keep POSIX numbering (by order of `(`), nested included;
        // escaped parens and bracket contents are not groups.
        assert_eq!(to_regex_ere_fancy("(a(X)b)\\2").unwrap(), "(a(X)b)\\2");
        assert_eq!(to_regex_ere_fancy("a\\(b[(]c\\)").unwrap(), "a\\(b[(]c\\)");

        // The translated pattern must compile under fancy-regex and match.
        let re = fancy_regex::Regex::new(&to_regex_ere_fancy("(ab)\\1").unwrap()).unwrap();
        assert!(re.is_match("abab").unwrap());
        assert!(!re.is_match("abac").unwrap());
    }

    #[test]
    fn back_references_translate_for_fancy() {
        use super::to_regex_fancy;

        assert_eq!(to_regex_fancy("a\\(b\\)c\\1").unwrap(), "a(b)c\\1");
        assert_eq!(to_regex_fancy("\\(ab\\)\\1").unwrap(), "(ab)\\1");
        // Groups keep POSIX numbering (by order of `\(`), nested included.
        assert_eq!(to_regex_fancy("\\(a\\(X\\)b\\)\\2").unwrap(), "(a(X)b)\\2");

        // The translated pattern must compile under fancy-regex and match.
        let re = fancy_regex::Regex::new(&to_regex_fancy("\\(ab\\)\\1").unwrap()).unwrap();
        let caps = re.captures("abab").unwrap().unwrap();
        assert_eq!(caps.get(1).map(|m| m.as_str()), Some("ab"));

        let re = fancy_regex::Regex::new(&to_regex_fancy("\\(a\\(X\\)b\\)\\2").unwrap()).unwrap();
        let caps = re.captures("aXbXb").unwrap().unwrap();
        assert_eq!(caps.get(1).map(|m| m.as_str()), Some("aXb"));
        assert_eq!(caps.get(2).map(|m| m.as_str()), Some("X"));
    }

    #[test]
    fn invalid_back_references_error() {
        use super::{to_regex_ere_fancy, to_regex_fancy};

        // GNU expr: "Invalid back reference" whenever the Nth group is not
        // closed before the back reference.
        assert!(to_regex_fancy("\\1").is_err());
        assert!(to_regex_fancy("\\(a\\)\\2").is_err());
        // A reference inside the group it refers to (or an enclosing group)
        // is invalid because that group is not closed yet.
        assert!(to_regex_fancy("\\(a\\1\\)").is_err());
        assert!(to_regex_fancy("\\(a\\(b\\)\\1\\)").is_err());
        // ... while a closed group referenced from inside later groups is fine.
        assert_eq!(
            to_regex_fancy("\\(a\\)\\(\\(b\\)\\1\\)").unwrap(),
            "(a)((b)\\1)"
        );

        // GNU sed -E applies the same rule to ERE back references.
        assert!(to_regex_ere_fancy("\\1(a)").is_err());
        assert!(to_regex_ere_fancy("(a)\\2").is_err());
        assert!(to_regex_ere_fancy("(a\\1)").is_err());
        // ... and a closed earlier group is referable from a later one.
        assert_eq!(to_regex_ere_fancy("(a)(b)\\1").unwrap(), "(a)(b)\\1");
    }

    #[test]
    fn unbalanced_groups_error() {
        assert!(to_regex("\\(a").is_err());
        assert!(to_regex("a\\)").is_err());
    }
}
