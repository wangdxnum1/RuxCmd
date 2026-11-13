//! Shared GNU size-suffix parsing (`K`/`KB`/`M`/`MB`/… with the 1024 and
//! 1000 bases) used by du --block-size, truncate -s and dd's numeric
//! operands.  The tables live here once so a fix lands everywhere.

/// The common coreutils block-size table: single-letter suffixes are
/// 1024-based, doubled-letter suffixes are 1000-based, through E/EB.
/// Matching is case-insensitive; the bare suffix means ×1.
pub fn multiplier_common(suffix_upper: &str) -> Option<u64> {
    match suffix_upper {
        "" => Some(1),
        "K" => Some(1024),
        "M" => Some(1024 * 1024),
        "G" => Some(1024 * 1024 * 1024),
        "T" => Some(1024u64 * 1024 * 1024 * 1024),
        "P" => Some(1024u64 * 1024 * 1024 * 1024 * 1024),
        "E" => Some(1024u64 * 1024 * 1024 * 1024 * 1024 * 1024),
        "KB" => Some(1000),
        "MB" => Some(1000 * 1000),
        "GB" => Some(1000 * 1000 * 1000),
        "TB" => Some(1000u64 * 1000 * 1000 * 1000),
        "PB" => Some(1000u64 * 1000 * 1000 * 1000 * 1000),
        "EB" => Some(1000u64 * 1000 * 1000 * 1000 * 1000 * 1000),
        _ => None,
    }
}

/// dd's extended table on top of the common one: `c` = 1 byte, `w` = 2,
/// `b` = 512, plus the IEC `kib`/`mib`/`gib` spellings (dd lowercases the
/// suffix before matching).
pub fn multiplier_dd(suffix_lower: &str) -> Option<u64> {
    match suffix_lower {
        "c" => Some(1),
        "w" => Some(2),
        "b" => Some(512),
        "kib" => Some(1024),
        "mib" => Some(1024 * 1024),
        "gib" => Some(1024 * 1024 * 1024),
        other => multiplier_common(&other.to_ascii_uppercase()),
    }
}

/// Parses `digits + optional suffix` with the common table; the whole spec
/// must be consumed.  Used by du --block-size and truncate -s.
pub fn parse_common(spec: &str) -> Option<u64> {
    let split = spec.find(|c: char| c.is_ascii_alphabetic());
    let (num_part, suffix) = match split {
        Some(idx) => (&spec[..idx], spec[idx..].to_ascii_uppercase()),
        None => (spec, String::new()),
    };
    let num: u64 = num_part.parse().ok()?;
    let multiplier = multiplier_common(&suffix)?;
    num.checked_mul(multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_table_covers_both_bases() {
        assert_eq!(multiplier_common(""), Some(1));
        assert_eq!(multiplier_common("K"), Some(1024));
        assert_eq!(multiplier_common("KB"), Some(1000));
        assert_eq!(multiplier_common("M"), Some(1024 * 1024));
        assert_eq!(multiplier_common("MB"), Some(1000 * 1000));
        assert_eq!(
            multiplier_common("EB"),
            Some(1000u64 * 1000 * 1000 * 1000 * 1000 * 1000)
        );
        assert_eq!(multiplier_common("X"), None);
        assert_eq!(multiplier_common("KIB"), None, "IEC spellings belong to dd");
    }

    #[test]
    fn common_parse_requires_the_whole_spec() {
        assert_eq!(parse_common("512"), Some(512));
        assert_eq!(parse_common("2K"), Some(2048));
        assert_eq!(parse_common("1KB"), Some(1000));
        assert_eq!(parse_common("1kb"), Some(1000), "case-insensitive");
        assert_eq!(parse_common("K"), None, "bare suffix needs a number here");
        assert_eq!(parse_common("2X"), None);
        assert_eq!(parse_common("2KB3"), None);
    }

    #[test]
    fn dd_table_adds_block_and_iec_units() {
        assert_eq!(multiplier_dd("c"), Some(1));
        assert_eq!(multiplier_dd("w"), Some(2));
        assert_eq!(multiplier_dd("b"), Some(512));
        assert_eq!(multiplier_dd("k"), Some(1024));
        assert_eq!(multiplier_dd("kib"), Some(1024));
        assert_eq!(multiplier_dd("mib"), Some(1024 * 1024));
        assert_eq!(multiplier_dd("gib"), Some(1024 * 1024 * 1024));
        assert_eq!(multiplier_dd("t"), Some(1024u64 * 1024 * 1024 * 1024));
        assert_eq!(multiplier_dd("tb"), Some(1000u64 * 1000 * 1000 * 1000));
        assert_eq!(multiplier_dd("zz"), None);
    }
}
