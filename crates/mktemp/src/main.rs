mod cli;

use clap::Parser;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use windows_sys::Win32::Security::Cryptography::{
    BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom,
};

/// GNU gen_tempname replaces the X's with characters from [a-zA-Z0-9].
const RANDOM_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
/// GNU requires at least 3 consecutive X's in the template.
const MIN_X_COUNT: usize = 3;
/// Number of distinct random names tried before giving up.
const MAX_ATTEMPTS: usize = 100;

fn main() {
    let args = cli::Args::parse();

    let template = match args.templates.len() {
        0 => "tmp.XXXXXXXXXX".to_string(),
        1 => args.templates[0].clone(),
        _ => {
            eprintln!("mktemp: too many templates");
            std::process::exit(1);
        }
    };

    // No template operand (or an explicit --tmpdir): the name is resolved
    // inside a temporary directory. GNU uses $TMPDIR or /tmp; on Windows the
    // equivalent is %TEMP%.
    let use_dest_dir = args.templates.is_empty() || args.tmpdir.is_some() || args.use_temp_dir;

    let suffix = match &args.suffix {
        Some(sfx) => {
            if !template.ends_with('X') {
                eprintln!("with --suffix, template '{}' must end in X", template);
                std::process::exit(1);
            }
            if sfx.contains(['/', '\\']) {
                eprintln!("invalid suffix '{}', contains directory separator", sfx);
                std::process::exit(1);
            }
            sfx.clone()
        }
        None => {
            // Implicit suffix: everything after the last X in the template
            // (GNU splits the template the same way when it does not end in
            // X, so `fooXXXbar` randomizes the XXX and keeps `bar`).
            match template.rfind('X') {
                Some(pos) => {
                    let tail = template[pos + 1..].to_string();
                    if tail.contains(['/', '\\']) {
                        eprintln!("invalid suffix '{}', contains directory separator", tail);
                        std::process::exit(1);
                    }
                    tail
                }
                None => String::new(),
            }
        }
    };

    // Count the run of X's that ends immediately before the suffix: for an
    // explicit --suffix the X-run reaches the end of the template itself
    // (the suffix is appended afterwards); for an implicit suffix the tail
    // is still inside the template and the X-run ends right before it.
    let explicit_suffix = args.suffix.is_some();
    let suffix_len = suffix.len();
    let boundary = if explicit_suffix {
        template.len()
    } else {
        template.len() - suffix_len
    };
    let template_bytes = template.as_bytes();
    let mut x_count = 0usize;
    while x_count < boundary && template_bytes[boundary - 1 - x_count] == b'X' {
        x_count += 1;
    }
    if x_count < MIN_X_COUNT {
        eprintln!("mktemp: too few X's in template '{}'", template);
        std::process::exit(1);
    }

    if use_dest_dir {
        if Path::new(&template).components().any(|part| {
            matches!(
                part,
                std::path::Component::Prefix(_) | std::path::Component::RootDir
            )
        }) || (args.use_temp_dir && template.contains(['/', '\\']))
        {
            eprintln!(
                "mktemp: invalid template, '{}'; with --tmpdir, it may not be absolute",
                template
            );
            std::process::exit(1);
        }
        let dest_dir = match &args.tmpdir {
            Some(dir) => dir.clone(),
            None => std::env::temp_dir(),
        };
        // Path::join inserts the platform separator (GNU file_name_concat).
        let mut full = dest_dir.join(&template).into_os_string();
        if explicit_suffix {
            full.push(suffix.as_str());
        }
        create_and_print(&full, &args, suffix_len, x_count);
    } else {
        let mut full = std::ffi::OsString::from(template.as_str());
        if explicit_suffix {
            full.push(suffix.as_str());
        }
        create_and_print(&full, &args, suffix_len, x_count);
    }
}

fn create_and_print(
    full: &std::ffi::OsString,
    args: &cli::Args,
    suffix_len: usize,
    x_count: usize,
) {
    let full_str = full.to_string_lossy().into_owned();
    let mut last_err: Option<std::io::Error> = None;

    for _ in 0..MAX_ATTEMPTS {
        let mut name_bytes = full_str.clone().into_bytes();
        let x_start = name_bytes.len() - suffix_len - x_count;
        if let Err(error) = randomize(&mut name_bytes[x_start..x_start + x_count]) {
            fail(&full_str, args, error);
        }
        let candidate =
            PathBuf::from(String::from_utf8(name_bytes).expect("template stays valid UTF-8"));

        let result = if args.directory {
            fs::create_dir(&candidate).map(|_| ())
        } else {
            // create_new gives the O_EXCL semantics a temp-name race needs.
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&candidate)
                .map(|_| ())
        };

        match result {
            Ok(()) => {
                println!("{}", candidate.display());
                return;
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => last_err = Some(e),
            Err(e) => fail(&full_str, args, e),
        }
    }

    let e = last_err.unwrap_or_else(|| std::io::Error::other("too many attempts"));
    fail(&full_str, args, e);
}

fn fail(full_str: &str, args: &cli::Args, e: std::io::Error) -> ! {
    if !args.quiet {
        let kind = if args.directory { "directory" } else { "file" };
        eprintln!(
            "mktemp: failed to create {} via template '{}': {}",
            kind, full_str, e
        );
    }
    std::process::exit(1);
}

/// Draw unpredictable names from the Windows system RNG. Rejection sampling
/// keeps all 62 template characters equally likely.
fn randomize(slots: &mut [u8]) -> std::io::Result<()> {
    let mut filled = 0;
    while filled < slots.len() {
        let mut bytes = [0u8; 64];
        let status = unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                bytes.as_mut_ptr(),
                bytes.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };
        if status < 0 {
            return Err(std::io::Error::other(format!(
                "system random generator failed: {status:#x}"
            )));
        }
        for byte in bytes {
            if byte < 248 {
                slots[filled] = RANDOM_CHARS[byte as usize % RANDOM_CHARS.len()];
                filled += 1;
                if filled == slots.len() {
                    break;
                }
            }
        }
    }
    Ok(())
}
