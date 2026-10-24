mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read, Write};

/// Input is consumed 64 KiB at a time; scanners keep cross-chunk state so
/// printable sequences (and their start offsets) survive chunk boundaries.
const CHUNK: usize = 64 * 1024;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("strings 0.1.0");
        return;
    }

    let min_length = args.min_length;
    let encoding = args.encoding;
    let files = args.files;
    let print_file_name = args.print_file_name;
    let radix = args.radix;

    let mut exit_code = 0i32;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    if files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = strings_reader(
            stdin.lock(),
            &mut out,
            min_length,
            encoding,
            None,
            print_file_name,
            radix,
        ) {
            eprintln!("strings: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for path in &files {
            let path_str = path.to_string_lossy();
            let file_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("(stdin)");
            if path_str == "-" {
                let stdin = std::io::stdin();
                if let Err(e) = strings_reader(
                    stdin.lock(),
                    &mut out,
                    min_length,
                    encoding,
                    None,
                    print_file_name,
                    radix,
                ) {
                    eprintln!("strings: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = strings_reader(
                            reader,
                            &mut out,
                            min_length,
                            encoding,
                            Some(file_name),
                            print_file_name,
                            radix,
                        ) {
                            eprintln!("strings: {}: {}", path.display(), e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("strings: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    let _ = out.flush();
    std::process::exit(exit_code);
}

fn format_offset(offset: usize, radix: &cli::Radix) -> String {
    match radix {
        cli::Radix::Octal => format!("{:07o}", offset),
        cli::Radix::Hexadecimal => format!("{:06x}", offset),
        cli::Radix::Decimal => format!("{:08}", offset),
    }
}

fn is_printable_seven_bit(byte: u8) -> bool {
    byte >= 0x20 && byte <= 0x7E
}

fn is_printable_eight_bit(byte: u8) -> bool {
    byte >= 0x20 && byte <= 0xFE
}

fn strings_reader<R: Read, W: Write>(
    mut reader: R,
    out: &mut W,
    min_length: usize,
    encoding: Option<cli::Encoding>,
    file_name: Option<&str>,
    print_file_name: bool,
    radix: Option<cli::Radix>,
) -> Result<(), std::io::Error> {
    let encoding = encoding.unwrap_or(cli::Encoding::SevenBit);

    match encoding {
        cli::Encoding::SevenBit => scan_bytes(
            &mut reader,
            out,
            is_printable_seven_bit,
            min_length,
            file_name,
            print_file_name,
            radix,
        ),
        cli::Encoding::EightBit => scan_bytes(
            &mut reader,
            out,
            is_printable_eight_bit,
            min_length,
            file_name,
            print_file_name,
            radix,
        ),
        cli::Encoding::Utf16BE => scan_utf16(
            &mut reader,
            out,
            false,
            min_length,
            file_name,
            print_file_name,
            radix,
        ),
        cli::Encoding::Utf16LE => scan_utf16(
            &mut reader,
            out,
            true,
            min_length,
            file_name,
            print_file_name,
            radix,
        ),
        cli::Encoding::Utf32BE => scan_utf32(
            &mut reader,
            out,
            false,
            min_length,
            file_name,
            print_file_name,
            radix,
        ),
        cli::Encoding::Utf32LE => scan_utf32(
            &mut reader,
            out,
            true,
            min_length,
            file_name,
            print_file_name,
            radix,
        ),
    }
}

/// Streaming scanner for the single-byte encodings (`s` / `S`): printable
/// runs are accumulated across chunk boundaries and flushed at separators,
/// non-printables or EOF. Offsets are absolute over the whole input.
fn scan_bytes<R: Read, W: Write>(
    reader: &mut R,
    out: &mut W,
    is_printable: fn(u8) -> bool,
    min_length: usize,
    file_name: Option<&str>,
    print_file_name: bool,
    radix: Option<cli::Radix>,
) -> Result<(), std::io::Error> {
    let mut chunk = vec![0u8; CHUNK];
    let mut current_string: Vec<u8> = Vec::new();
    let mut start_offset = 0usize;
    let mut pos = 0usize; // absolute offset of the next byte to inspect

    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        for &byte in &chunk[..n] {
            if is_printable(byte) || byte == b'\n' || byte == b'\r' {
                if byte == b'\n' || byte == b'\r' {
                    if current_string.len() >= min_length {
                        print_result(
                            out,
                            file_name,
                            print_file_name,
                            radix,
                            start_offset,
                            &String::from_utf8_lossy(&current_string),
                        )?;
                    }
                    current_string.clear();
                } else {
                    if current_string.is_empty() {
                        start_offset = pos;
                    }
                    current_string.push(byte);
                }
            } else if current_string.len() >= min_length {
                print_result(
                    out,
                    file_name,
                    print_file_name,
                    radix,
                    start_offset,
                    &String::from_utf8_lossy(&current_string),
                )?;
                current_string.clear();
            } else {
                current_string.clear();
            }
            pos += 1;
        }
    }

    if current_string.len() >= min_length {
        print_result(
            out,
            file_name,
            print_file_name,
            radix,
            start_offset,
            &String::from_utf8_lossy(&current_string),
        )?;
    }
    Ok(())
}

/// Streaming scanner for UTF-16 (`b` / `l`): a 0-1 byte carry keeps partial
/// units across chunk boundaries; `base` is the absolute offset of the
/// first carry byte so unit offsets stay continuous.
fn scan_utf16<R: Read, W: Write>(
    reader: &mut R,
    out: &mut W,
    little_endian: bool,
    min_length: usize,
    file_name: Option<&str>,
    print_file_name: bool,
    radix: Option<cli::Radix>,
) -> Result<(), std::io::Error> {
    let mut chunk = vec![0u8; CHUNK];
    let mut carry = [0u8; 1];
    let mut carry_len = 0usize;
    let mut base = 0usize; // absolute offset of carry[0]
    let mut current_string: Vec<u16> = Vec::new();
    let mut start_offset = 0usize;

    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        let mut buf: Vec<u8> = Vec::with_capacity(carry_len + n);
        buf.extend_from_slice(&carry[..carry_len]);
        buf.extend_from_slice(&chunk[..n]);

        let units = buf.len() / 2;
        for u in 0..units {
            let i = 2 * u;
            let ch = if little_endian {
                ((buf[i + 1] as u32) << 8) | (buf[i] as u32)
            } else {
                ((buf[i] as u32) << 8) | (buf[i + 1] as u32)
            };
            let offset = base + i;
            if ch == 0x000A || ch == 0x000D {
                if current_string.len() >= min_length {
                    print_result(
                        out,
                        file_name,
                        print_file_name,
                        radix,
                        start_offset,
                        &String::from_utf16_lossy(&current_string),
                    )?;
                }
                current_string.clear();
            } else if is_printable_unicode(ch) {
                if current_string.is_empty() {
                    start_offset = offset;
                }
                current_string.push(ch as u16);
            } else if current_string.len() >= min_length {
                print_result(
                    out,
                    file_name,
                    print_file_name,
                    radix,
                    start_offset,
                    &String::from_utf16_lossy(&current_string),
                )?;
                current_string.clear();
            } else {
                current_string.clear();
            }
        }

        let consumed = units * 2;
        carry_len = buf.len() - consumed;
        carry[..carry_len].copy_from_slice(&buf[consumed..]);
        base += consumed;
    }

    if current_string.len() >= min_length {
        print_result(
            out,
            file_name,
            print_file_name,
            radix,
            start_offset,
            &String::from_utf16_lossy(&current_string),
        )?;
    }
    Ok(())
}

/// Streaming scanner for UTF-32 (`B` / `L`): same carry scheme with up to 3
/// leftover bytes held across chunk boundaries.
fn scan_utf32<R: Read, W: Write>(
    reader: &mut R,
    out: &mut W,
    little_endian: bool,
    min_length: usize,
    file_name: Option<&str>,
    print_file_name: bool,
    radix: Option<cli::Radix>,
) -> Result<(), std::io::Error> {
    let mut chunk = vec![0u8; CHUNK];
    let mut carry = [0u8; 3];
    let mut carry_len = 0usize;
    let mut base = 0usize; // absolute offset of carry[0]
    let mut current_string = String::new();
    let mut start_offset = 0usize;

    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        let mut buf: Vec<u8> = Vec::with_capacity(carry_len + n);
        buf.extend_from_slice(&carry[..carry_len]);
        buf.extend_from_slice(&chunk[..n]);

        let units = buf.len() / 4;
        for u in 0..units {
            let i = 4 * u;
            let ch = if little_endian {
                ((buf[i + 3] as u32) << 24)
                    | ((buf[i + 2] as u32) << 16)
                    | ((buf[i + 1] as u32) << 8)
                    | (buf[i] as u32)
            } else {
                ((buf[i] as u32) << 24)
                    | ((buf[i + 1] as u32) << 16)
                    | ((buf[i + 2] as u32) << 8)
                    | (buf[i + 3] as u32)
            };
            let offset = base + i;
            if ch == 0x0000000A || ch == 0x0000000D {
                if current_string.len() >= min_length {
                    print_result(
                        out,
                        file_name,
                        print_file_name,
                        radix,
                        start_offset,
                        &current_string,
                    )?;
                }
                current_string.clear();
            } else if is_printable_unicode(ch) {
                if current_string.is_empty() {
                    start_offset = offset;
                }
                if let Some(c) = std::char::from_u32(ch) {
                    current_string.push(c);
                } else if current_string.len() >= min_length {
                    print_result(
                        out,
                        file_name,
                        print_file_name,
                        radix,
                        start_offset,
                        &current_string,
                    )?;
                    current_string.clear();
                } else {
                    current_string.clear();
                }
            } else if current_string.len() >= min_length {
                print_result(
                    out,
                    file_name,
                    print_file_name,
                    radix,
                    start_offset,
                    &current_string,
                )?;
                current_string.clear();
            } else {
                current_string.clear();
            }
        }

        let consumed = units * 4;
        carry_len = buf.len() - consumed;
        carry[..carry_len].copy_from_slice(&buf[consumed..]);
        base += consumed;
    }

    if current_string.len() >= min_length {
        print_result(
            out,
            file_name,
            print_file_name,
            radix,
            start_offset,
            &current_string,
        )?;
    }
    Ok(())
}

fn is_printable_unicode(ch: u32) -> bool {
    if ch == 0x0020 {
        return true;
    }

    let c = match std::char::from_u32(ch) {
        Some(c) => c,
        None => return false,
    };

    c.is_alphanumeric()
        || c.is_ascii_punctuation()
        || c.is_whitespace()
        || (ch >= 0x20 && ch <= 0x7E)
}

fn print_result<W: Write>(
    out: &mut W,
    file_name: Option<&str>,
    print_file_name: bool,
    radix: Option<cli::Radix>,
    offset: usize,
    s: &str,
) -> Result<(), std::io::Error> {
    if print_file_name && file_name.is_some() {
        write!(out, "{}:", file_name.unwrap())?;
    }
    if let Some(r) = radix {
        write!(out, "{} ", format_offset(offset, &r))?;
    }
    writeln!(out, "{}", s)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Reader that hands out exactly one byte per `read` call: every possible
    /// chunk split (including mid-UTF-16/32-unit splits) is exercised.
    struct OneByteAtATime<R>(R);

    impl<R: Read> Read for OneByteAtATime<R> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
            self.0.read(&mut buf[..1])
        }
    }

    fn scan(
        input: &[u8],
        encoding: cli::Encoding,
        min_length: usize,
        radix: Option<cli::Radix>,
    ) -> String {
        let mut out = Vec::new();
        strings_reader(
            OneByteAtATime(Cursor::new(input.to_vec())),
            &mut out,
            min_length,
            Some(encoding),
            None,
            false,
            radix,
        )
        .unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn seven_bit_scan_survives_byte_at_a_time_reads() {
        assert_eq!(
            scan(b"\x01hello world\x00abcd", cli::Encoding::SevenBit, 4, None),
            "hello world\nabcd\n"
        );
        // \n flushes a pending string even mid-input.
        assert_eq!(
            scan(b"ab\x00wxyz\nmore\x00", cli::Encoding::SevenBit, 4, None),
            "wxyz\nmore\n"
        );
    }

    #[test]
    fn seven_bit_offset_is_absolute() {
        assert_eq!(
            scan(
                b"\x00\x00wxyz\n",
                cli::Encoding::SevenBit,
                4,
                Some(cli::Radix::Decimal)
            ),
            "00000002 wxyz\n"
        );
    }

    #[test]
    fn eight_bit_scan_passes_high_bytes() {
        // 0x80..0x83 are printable in the 8-bit (S) mode and render as U+FFFD
        // through the lossy conversion, exactly like the buffered version.
        assert_eq!(
            scan(b"\x80\x81\x82\x83\x00", cli::Encoding::EightBit, 4, None),
            format!("{}\n", "\u{FFFD}".repeat(4))
        );
    }

    #[test]
    fn utf16_be_and_le_survive_unit_splits() {
        // "hi!!" as UTF-16 units, padded by NUL units (non-printable).
        let be: Vec<u8> = [
            0x00u16,
            b'h' as u16,
            b'i' as u16,
            b'!' as u16,
            b'!' as u16,
            0x0000,
        ]
        .iter()
        .flat_map(|u| u.to_be_bytes())
        .collect();
        let le: Vec<u8> = [
            0x00u16,
            b'h' as u16,
            b'i' as u16,
            b'!' as u16,
            b'!' as u16,
            0x0000,
        ]
        .iter()
        .flat_map(|u| u.to_le_bytes())
        .collect();
        assert_eq!(scan(&be, cli::Encoding::Utf16BE, 4, None), "hi!!\n");
        assert_eq!(scan(&le, cli::Encoding::Utf16LE, 4, None), "hi!!\n");
    }

    #[test]
    fn utf16_trailing_odd_byte_is_ignored() {
        let mut le: Vec<u8> = [b'w' as u16, b'x' as u16, b'y' as u16, b'z' as u16]
            .iter()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        le.push(0xFF); // odd trailing byte: dropped, exactly like read_to_end
        assert_eq!(scan(&le, cli::Encoding::Utf16LE, 4, None), "wxyz\n");
    }

    #[test]
    fn utf32_be_and_le_survive_unit_splits() {
        let be: Vec<u8> = [
            0x00000000u32,
            b'k' as u32,
            b'o' as u32,
            b'!' as u32,
            b'!' as u32,
        ]
        .iter()
        .flat_map(|u| u.to_be_bytes())
        .collect();
        let le: Vec<u8> = [
            0x00000000u32,
            b'k' as u32,
            b'o' as u32,
            b'!' as u32,
            b'!' as u32,
        ]
        .iter()
        .flat_map(|u| u.to_le_bytes())
        .collect();
        assert_eq!(scan(&be, cli::Encoding::Utf32BE, 4, None), "ko!!\n");
        assert_eq!(scan(&le, cli::Encoding::Utf32LE, 4, None), "ko!!\n");
    }
}
