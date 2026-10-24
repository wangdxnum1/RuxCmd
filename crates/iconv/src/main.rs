mod cli;

use clap::Parser;
use encoding_rs::{CoderResult, Encoding};
use std::fs;
use std::io::{Read, Write};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("iconv 0.1.0");
        return;
    }

    if args.list {
        list_encodings();
        return;
    }

    let from_code = args.from_code.as_deref().unwrap_or("");
    let to_code = args.to_code.as_deref().unwrap_or("");

    if from_code.is_empty() || to_code.is_empty() {
        eprintln!("iconv: missing from-code or to-code");
        std::process::exit(1);
    }

    let from_encoding = match Encoding::for_label(from_code.as_bytes()) {
        Some(e) => e,
        None => {
            eprintln!("iconv: invalid from-code '{}'", from_code);
            std::process::exit(1);
        }
    };

    let to_encoding = match Encoding::for_label(to_code.as_bytes()) {
        Some(e) => e,
        None => {
            eprintln!("iconv: invalid to-code '{}'", to_code);
            std::process::exit(1);
        }
    };

    let mut exit_code = 0i32;

    let mut all_input = Vec::new();

    if args.files.is_empty() {
        if let Err(e) = std::io::stdin().read_to_end(&mut all_input) {
            eprintln!("iconv: error reading stdin: {}", e);
            std::process::exit(1);
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                if let Err(e) = std::io::stdin().read_to_end(&mut all_input) {
                    eprintln!("iconv: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match fs::read(path) {
                    Ok(data) => all_input.extend(data),
                    Err(e) => {
                        eprintln!("iconv: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    // Decode strictly according to -f: no BOM sniffing and no BOM removal.
    // A UTF-8 BOM on the input of `iconv -f gbk` must be decoded as GBK
    // bytes, exactly like GNU iconv does.
    let decoded = decode_strict(from_encoding, &all_input);
    let (output_bytes, _, _) = to_encoding.encode(&decoded);

    if let Some(out_path) = &args.output {
        if let Err(e) = fs::write(out_path, &output_bytes) {
            eprintln!("iconv: {}: {}", out_path.display(), e);
            std::process::exit(1);
        }
    } else {
        if let Err(e) = std::io::stdout().write_all(&output_bytes) {
            eprintln!("iconv: write error: {}", e);
            std::process::exit(1);
        }
    }

    std::process::exit(exit_code);
}

/// Decode the whole input with the given encoding, without BOM sniffing.
/// Malformed sequences are replaced with U+FFFD (same as the previous
/// `Encoding::decode` path, minus the heuristic).
fn decode_strict(encoding: &'static Encoding, input: &[u8]) -> String {
    let mut decoder = encoding.new_decoder_without_bom_handling();
    // Worst-case UTF-8 growth is 3 bytes per input byte.
    let mut decoded = String::with_capacity(input.len() * 3 + 4);
    let mut src = input;
    loop {
        let (result, read, _) = decoder.decode_to_string(src, &mut decoded, true);
        src = &src[read..];
        match result {
            CoderResult::InputEmpty => break,
            CoderResult::OutputFull => {
                decoded.reserve(src.len() * 3 + 4);
            }
        }
    }
    decoded
}

fn list_encodings() {
    let encodings: [(&Encoding, &str); 39] = [
        (encoding_rs::UTF_8, "Unicode (UTF-8)"),
        (encoding_rs::UTF_16LE, "Unicode (UTF-16 little-endian)"),
        (encoding_rs::UTF_16BE, "Unicode (UTF-16 big-endian)"),
        (encoding_rs::GBK, "GBK (Chinese)"),
        (encoding_rs::GB18030, "GB18030 (Chinese)"),
        (encoding_rs::BIG5, "Big5 (Traditional Chinese)"),
        (encoding_rs::EUC_JP, "EUC-JP (Japanese)"),
        (encoding_rs::SHIFT_JIS, "Shift_JIS (Japanese)"),
        (encoding_rs::ISO_2022_JP, "ISO-2022-JP (Japanese)"),
        (encoding_rs::EUC_KR, "EUC-KR (Korean)"),
        (encoding_rs::WINDOWS_874, "Windows-874 (Thai)"),
        (encoding_rs::WINDOWS_1250, "Windows-1250 (Central European)"),
        (encoding_rs::WINDOWS_1251, "Windows-1251 (Cyrillic)"),
        (encoding_rs::WINDOWS_1252, "Windows-1252 (Western European)"),
        (encoding_rs::WINDOWS_1253, "Windows-1253 (Greek)"),
        (encoding_rs::WINDOWS_1254, "Windows-1254 (Turkish)"),
        (encoding_rs::WINDOWS_1255, "Windows-1255 (Hebrew)"),
        (encoding_rs::WINDOWS_1256, "Windows-1256 (Arabic)"),
        (encoding_rs::WINDOWS_1257, "Windows-1257 (Baltic)"),
        (encoding_rs::WINDOWS_1258, "Windows-1258 (Vietnamese)"),
        (encoding_rs::IBM866, "IBM866 (Cyrillic)"),
        (encoding_rs::ISO_8859_2, "ISO-8859-2 (Latin-2)"),
        (encoding_rs::ISO_8859_3, "ISO-8859-3 (Latin-3)"),
        (encoding_rs::ISO_8859_4, "ISO-8859-4 (Latin-4)"),
        (encoding_rs::ISO_8859_5, "ISO-8859-5 (Cyrillic)"),
        (encoding_rs::ISO_8859_6, "ISO-8859-6 (Arabic)"),
        (encoding_rs::ISO_8859_7, "ISO-8859-7 (Greek)"),
        (encoding_rs::ISO_8859_8, "ISO-8859-8 (Hebrew)"),
        (encoding_rs::ISO_8859_8_I, "ISO-8859-8-I (Hebrew logical)"),
        (encoding_rs::ISO_8859_10, "ISO-8859-10 (Latin-6)"),
        (encoding_rs::ISO_8859_13, "ISO-8859-13 (Latin-7)"),
        (encoding_rs::ISO_8859_14, "ISO-8859-14 (Latin-8)"),
        (encoding_rs::ISO_8859_15, "ISO-8859-15 (Latin-9)"),
        (encoding_rs::ISO_8859_16, "ISO-8859-16 (Latin-10)"),
        (encoding_rs::KOI8_R, "KOI8-R (Cyrillic)"),
        (encoding_rs::KOI8_U, "KOI8-U (Cyrillic Ukrainian)"),
        (encoding_rs::MACINTOSH, "Macintosh (Roman)"),
        (encoding_rs::X_MAC_CYRILLIC, "Mac-Cyrillic"),
        (encoding_rs::X_USER_DEFINED, "X-User-Defined"),
    ];

    for (encoding, desc) in encodings {
        println!("{} - {}", encoding.name(), desc);
    }
    // The bare "UTF-16" label sniffs the byte order from a BOM.
    println!("UTF-16 - Unicode (UTF-16 with BOM)");
}
