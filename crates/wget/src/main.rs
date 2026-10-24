mod cli;

use clap::Parser;
use reqwest::blocking::Client;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

/// Marker used for the display name when -O - writes to stdout.
const STDOUT_NAME: &str = "STDOUT";

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("wget 0.1.0");
        return;
    }

    let url = args.url.as_deref().unwrap_or_else(|| {
        eprintln!("wget: missing URL");
        std::process::exit(1);
    });

    let to_stdout = args
        .output
        .as_ref()
        .is_some_and(|p| p.as_os_str() == OsStr::new("-"));
    let output_path = if to_stdout {
        None
    } else {
        Some(resolve_output_path(&args, url))
    };
    let display_name = match &output_path {
        Some(path) => path.display().to_string(),
        None => STDOUT_NAME.to_string(),
    };

    if !args.quiet {
        println!("Connecting to {}...", url);
    }

    // Without a connect timeout a dead host could hang forever; the total
    // download intentionally has no limit (streamed downloads can be long).
    let client = match Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("wget: client build failed: {}", e);
            std::process::exit(1);
        }
    };

    let response = match client.get(url).send() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("wget: connection failed: {}", e);
            std::process::exit(1);
        }
    };

    let status = response.status();
    if !status.is_success() {
        // GNU wget exits 8 on server-issued errors.
        eprintln!(
            "wget: HTTP error {}: {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("unknown")
        );
        std::process::exit(8);
    }

    let content = match response.bytes() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("wget: read error: {}", e);
            std::process::exit(1);
        }
    };

    if !args.quiet {
        println!(
            "HTTP request sent, awaiting response... {} {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("OK")
        );
        println!("Length: {} [binary]", content.len());
        println!("Saving to: '{}'", display_name);
        println!();
    }

    if to_stdout {
        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        if let Err(e) = lock.write_all(&content) {
            eprintln!("wget: write error: {}", e);
            std::process::exit(1);
        }
        let _ = lock.flush();
    } else if let Some(path) = &output_path
        && let Err(e) = fs::write(path, &content)
    {
        eprintln!("wget: cannot write to {}: {}", path.display(), e);
        std::process::exit(1);
    }

    if !args.quiet {
        println!("'{}' saved", display_name);
    }
}

fn resolve_output_path(args: &cli::Args, url: &str) -> PathBuf {
    if let Some(o) = &args.output {
        return o.clone();
    }

    let filename = filename_from_url(url);
    let mut result = PathBuf::from(filename);

    if let Some(d) = &args.directory {
        result = d.join(result);
        if let Err(e) = fs::create_dir_all(d) {
            eprintln!("wget: cannot create directory {}: {}", d.display(), e);
            std::process::exit(1);
        }
    }

    result
}

/// Derive the output file name from the URL path, matching GNU wget:
/// - the query string and fragment are stripped,
/// - percent escapes in the last path segment are decoded,
/// - a URL whose path is empty or ends in "/" (directory listing) saves
///   as index.html.
fn filename_from_url(url: &str) -> String {
    // Take the path portion (after scheme://host); without a scheme treat
    // the whole string as a path.
    let path = match url.find("://") {
        Some(i) => {
            let rest = &url[i + 3..];
            match rest.find('/') {
                Some(j) => &rest[j..],
                None => "/",
            }
        }
        None => url,
    };

    let path = path.split(['?', '#']).next().unwrap_or("");
    if path.is_empty() || path.ends_with('/') {
        return "index.html".to_string();
    }

    let last = path.rsplit('/').next().unwrap_or("");
    if last.is_empty() {
        return "index.html".to_string();
    }

    let decoded = decode_percent(last);
    if decoded.is_empty() {
        "index.html".to_string()
    } else {
        decoded
    }
}

/// Decode %XX escapes (bytes, so UTF-8 sequences survive); malformed
/// escapes are kept literally.
fn decode_percent(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                out.push((hi * 16 + lo) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trailing_slash_uses_index_html() {
        assert_eq!(filename_from_url("https://example.com/"), "index.html");
        assert_eq!(filename_from_url("https://example.com/dir/"), "index.html");
    }

    #[test]
    fn host_only_url_uses_index_html() {
        assert_eq!(filename_from_url("https://example.com"), "index.html");
    }

    #[test]
    fn normal_url_uses_last_path_segment() {
        assert_eq!(
            filename_from_url("https://example.com/dir/file.txt"),
            "file.txt"
        );
        assert_eq!(
            filename_from_url("https://example.com/file.zip"),
            "file.zip"
        );
    }

    #[test]
    fn query_string_is_stripped() {
        assert_eq!(
            filename_from_url("https://example.com/file.txt?a=1&b=2"),
            "file.txt"
        );
        assert_eq!(filename_from_url("https://example.com/?q=x"), "index.html");
    }

    #[test]
    fn fragment_is_stripped() {
        assert_eq!(
            filename_from_url("https://example.com/file.txt#top"),
            "file.txt"
        );
    }

    #[test]
    fn percent_encoding_is_decoded() {
        assert_eq!(
            filename_from_url("https://example.com/a%20b.txt"),
            "a b.txt"
        );
        assert_eq!(
            filename_from_url("https://example.com/%E4%B8%AD.txt"),
            "中.txt"
        );
    }

    #[test]
    fn invalid_percent_sequences_are_kept_literally() {
        assert_eq!(
            filename_from_url("https://example.com/100%.txt"),
            "100%.txt"
        );
        assert_eq!(filename_from_url("https://example.com/%2z.txt"), "%2z.txt");
    }

    #[test]
    fn schemeless_input_is_treated_as_path() {
        assert_eq!(filename_from_url("file.txt"), "file.txt");
    }
}
