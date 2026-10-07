mod cli;

use clap::Parser;
use cli::Args;
use std::process::Command;

/// Quotes `s` for safe embedding in a PowerShell command string: wraps it in
/// single quotes and doubles any embedded single quote. Inside single quotes
/// PowerShell performs no interpolation, so spaces, `;`, `$()`, `&`, `%`, `"`
/// and non-ASCII characters all stay literal.
fn escape_ps_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

fn main() {
    let args = Args::parse();

    let zipfile = escape_ps_single_quote(&args.zipfile.to_string_lossy());
    let files: Vec<String> = args
        .files
        .iter()
        .map(|p| escape_ps_single_quote(p.to_str().unwrap_or("")))
        .collect();

    let recurse_flag = if args.recurse { "-r" } else { "" };

    let result = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "Compress-Archive {} -Path {} -DestinationPath {} -Force",
                recurse_flag,
                files.join(", "),
                zipfile
            ),
        ])
        .output();

    match result {
        Ok(output) => {
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
            std::process::exit(output.status.code().unwrap_or(1));
        }
        Err(e) => {
            eprintln!("zip: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::escape_ps_single_quote;

    #[test]
    fn wraps_plain_name_in_single_quotes() {
        assert_eq!(escape_ps_single_quote("file.txt"), "'file.txt'");
    }

    #[test]
    fn keeps_spaces_literal() {
        assert_eq!(escape_ps_single_quote("my file.txt"), "'my file.txt'");
        assert_eq!(escape_ps_single_quote("my out.zip"), "'my out.zip'");
    }

    #[test]
    fn doubles_embedded_single_quotes() {
        assert_eq!(escape_ps_single_quote("it's.txt"), "'it''s.txt'");
        assert_eq!(escape_ps_single_quote("a'b'c.txt"), "'a''b''c.txt'");
    }

    #[test]
    fn keeps_metacharacters_literal() {
        // Before the fix each of these could terminate or inject into the
        // unquoted command string; inside single quotes they are inert.
        assert_eq!(escape_ps_single_quote("a;b.txt"), "'a;b.txt'");
        assert_eq!(escape_ps_single_quote("$(calc).txt"), "'$(calc).txt'");
        assert_eq!(escape_ps_single_quote("a&b.txt"), "'a&b.txt'");
        assert_eq!(escape_ps_single_quote("a%b.txt"), "'a%b.txt'");
        // `"` needs no doubling inside a single-quoted string (and is illegal
        // in NTFS file names anyway); it must stay a literal character.
        assert_eq!(escape_ps_single_quote("a\"b.txt"), "'a\"b.txt'");
    }

    #[test]
    fn keeps_non_ascii_literal() {
        assert_eq!(escape_ps_single_quote("文件 名.txt"), "'文件 名.txt'");
    }

    #[test]
    fn combines_all_hostile_characters() {
        assert_eq!(
            escape_ps_single_quote("it's; $(calc) & 100%.txt"),
            "'it''s; $(calc) & 100%.txt'"
        );
    }
}
