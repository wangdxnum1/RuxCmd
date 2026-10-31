use clap::Parser;
use std::ffi::{OsStr, OsString};

/// Stream editor for filtering and transforming text — a Windows port of the Linux sed command.
///
/// Supported subset: `s/pat/rep/[g]`, `p`, `d`, `q` commands with optional
/// addresses (line number, `$`, `/substring/`, and `addr,addr` ranges).
/// Matching uses POSIX BRE by default, or ERE with -E.
#[derive(Parser, Debug)]
#[command(name = "sed", version, about, disable_version_flag = true)]
pub struct Args {
    /// Add the script to the commands to be executed
    #[arg(short = 'e', long = "expression", value_name = "SCRIPT")]
    pub expressions: Vec<String>,

    /// Read the script from FILE
    #[arg(short = 'f', long = "file", value_name = "SCRIPT_FILE")]
    pub script_files: Vec<String>,

    /// Suppress automatic printing of pattern space
    #[arg(short = 'n', long = "quiet", visible_alias = "silent")]
    pub quiet: bool,

    /// Edit files in place, rewriting through a temp file + rename;
    /// an attached SUFFIX (-i.bak / --in-place=.bak) keeps a backup copy
    #[arg(
        short = 'i',
        long = "in-place",
        value_name = "SUFFIX",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = ""
    )]
    pub in_place: Option<String>,

    /// Consider files as separate rather than as a single continuous stream
    #[arg(short = 's', long = "separate")]
    pub separate: bool,

    /// Use extended regular expressions (ERE) in scripts
    #[arg(short = 'E', short_alias = 'r', long = "regexp-extended")]
    pub extended: bool,

    /// The script (when no -e is given) followed by the file(s) to edit
    #[arg(value_name = "SCRIPT FILE...")]
    pub operands: Vec<String>,

    /// Show version information
    #[arg(short = 'V', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}

/// Parse the process arguments with GNU `sed -i[SUFFIX]` semantics: the
/// optional SUFFIX is only taken from an attached form (`-i.bak`), never
/// from the following argv element, so `sed -i 's/a/b/' file` keeps both
/// operands positional. clap's `require_equals` covers `-i`, `-i=.bak` and
/// `--in-place[=.bak]`; the attached short form is rewritten here first.
pub fn parse_args() -> Args {
    Args::parse_from(rewrite_attached_in_place(std::env::args_os()))
}

fn rewrite_attached_in_place(argv: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut out = Vec::new();
    let mut operands_only = false;
    for arg in argv {
        if operands_only {
            out.push(arg);
            continue;
        }
        if arg.as_encoded_bytes() == b"--" {
            operands_only = true;
            out.push(arg);
            continue;
        }
        let lossy = arg.to_string_lossy();
        if lossy.starts_with("--") {
            out.push(arg);
            continue;
        }
        if let Some(rest) = lossy.strip_prefix("-i") {
            if rest.is_empty() || rest.starts_with('=') {
                out.push(arg);
            } else {
                let mut rewritten = OsString::from("--in-place=");
                rewritten.push(OsStr::new(rest));
                out.push(rewritten);
            }
        } else {
            out.push(arg);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Args {
        Args::parse_from(rewrite_attached_in_place(args.iter().map(OsString::from)))
    }

    #[test]
    fn bare_in_place_has_empty_suffix() {
        let args = parse(&["sed", "-i", "s/a/b/", "f.txt"]);
        assert_eq!(args.in_place.as_deref(), Some(""));
        assert_eq!(args.operands, vec!["s/a/b/", "f.txt"]);
    }

    #[test]
    fn attached_short_in_place_suffix() {
        let args = parse(&["sed", "-i.bak", "s/a/b/", "f.txt"]);
        assert_eq!(args.in_place.as_deref(), Some(".bak"));
        assert_eq!(args.operands, vec!["s/a/b/", "f.txt"]);
    }

    #[test]
    fn long_in_place_with_equals_suffix() {
        let args = parse(&["sed", "--in-place=.orig", "s/a/b/", "f.txt"]);
        assert_eq!(args.in_place.as_deref(), Some(".orig"));
        assert_eq!(args.operands, vec!["s/a/b/", "f.txt"]);
    }

    #[test]
    fn rewrite_converts_attached_short_form() {
        let argv: Vec<OsString> = ["sed", "-i.bak", "x"].iter().map(OsString::from).collect();
        let rewritten = rewrite_attached_in_place(argv);
        assert_eq!(
            rewritten,
            vec![
                OsString::from("sed"),
                OsString::from("--in-place=.bak"),
                OsString::from("x")
            ]
        );
    }

    #[test]
    fn rewrite_keeps_bare_and_equals_forms() {
        let argv: Vec<OsString> = ["sed", "-i", "-i=.b", "--", "-i.bak"]
            .iter()
            .map(OsString::from)
            .collect();
        let rewritten = rewrite_attached_in_place(argv);
        assert_eq!(
            rewritten,
            vec![
                OsString::from("sed"),
                OsString::from("-i"),
                OsString::from("-i=.b"),
                OsString::from("--"),
                OsString::from("-i.bak")
            ]
        );
    }

    #[test]
    fn quiet_flag_and_alias_parse() {
        let args = parse(&["sed", "-n", "2p"]);
        assert!(args.quiet);
        let args = parse(&["sed", "--silent", "2p"]);
        assert!(args.quiet);
    }
}
