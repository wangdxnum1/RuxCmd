mod cli;

use clap::Parser;
use encoding_rs::{CoderResult, Decoder, Encoder, Encoding};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

/// Input is decoded 8 KiB at a time and re-encoded immediately; neither the
/// whole input nor the whole output is ever held in memory.
const CHUNK: usize = 8 * 1024;

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

    // Decode strictly according to -f: no BOM sniffing and no BOM removal.
    // A UTF-8 BOM on the input of `iconv -f gbk` must be decoded as GBK
    // bytes, exactly like GNU iconv does.
    let mut converter = Converter::new(from_encoding, to_encoding);

    // The output is opened first so an unwritable -o target fails the same
    // way the previous fs::write-based version did.
    let exit_code = if let Some(out_path) = &args.output {
        let file = match File::create(out_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("iconv: {}: {}", out_path.display(), e);
                std::process::exit(1);
            }
        };
        let mut writer = BufWriter::new(file);
        let code = convert_files(&args, &mut converter, &mut writer);
        if let Err(e) = writer.flush() {
            eprintln!("iconv: {}: {}", out_path.display(), e);
            std::process::exit(1);
        }
        code
    } else {
        let stdout = std::io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        let code = convert_files(&args, &mut converter, &mut writer);
        if let Err(e) = writer.flush() {
            eprintln!("iconv: write error: {}", e);
            std::process::exit(1);
        }
        code
    };

    std::process::exit(exit_code);
}

/// Read every input (concatenated, like the old all_input Vec) and push it
/// through the converter. Returns the process exit code; write failures
/// abort the process directly, matching the previous single-shot writer.
fn convert_files<W: Write>(args: &cli::Args, converter: &mut Converter, writer: &mut W) -> i32 {
    fn report_write_error(args: &cli::Args, e: &std::io::Error) {
        match &args.output {
            Some(out_path) => eprintln!("iconv: {}: {}", out_path.display(), e),
            None => eprintln!("iconv: write error: {}", e),
        }
    }

    let mut in_buf = vec![0u8; CHUNK];
    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        let mut reader = stdin.lock();
        loop {
            match reader.read(&mut in_buf) {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(e) = converter.push(&in_buf[..n], writer) {
                        let _ = writer.flush();
                        report_write_error(args, &e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    let _ = writer.flush();
                    eprintln!("iconv: error reading stdin: {}", e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                let stdin = std::io::stdin();
                let mut reader = stdin.lock();
                loop {
                    match reader.read(&mut in_buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if let Err(e) = converter.push(&in_buf[..n], writer) {
                                let _ = writer.flush();
                                report_write_error(args, &e);
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("iconv: error reading stdin: {}", e);
                            exit_code = 1;
                            break;
                        }
                    }
                }
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let mut reader = BufReader::new(file);
                        loop {
                            match reader.read(&mut in_buf) {
                                Ok(0) => break,
                                Ok(n) => {
                                    if let Err(e) = converter.push(&in_buf[..n], writer) {
                                        let _ = writer.flush();
                                        report_write_error(args, &e);
                                        std::process::exit(1);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("iconv: {}: {}", path.display(), e);
                                    exit_code = 1;
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("iconv: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    if let Err(e) = converter.finish(writer) {
        let _ = writer.flush();
        report_write_error(args, &e);
        std::process::exit(1);
    }

    exit_code
}

/// Streaming -f -> -t conversion: bytes are decoded (strictly, without BOM
/// handling) and immediately re-encoded. The decoder keeps partial input
/// sequences across `push` calls, so a multi-byte character split over two
/// chunks still converts correctly; ditto for the encoder's state.
struct Converter {
    decoder: Decoder,
    encoder: Encoder,
    decoded: String,
    out_buf: Vec<u8>,
}

impl Converter {
    fn new(from: &'static Encoding, to: &'static Encoding) -> Self {
        Converter {
            decoder: from.new_decoder_without_bom_handling(),
            encoder: to.new_encoder(),
            decoded: String::with_capacity(CHUNK * 3),
            out_buf: Vec::with_capacity(CHUNK * 3),
        }
    }

    /// Feed one input chunk; converted bytes are written to `writer`.
    fn push<W: Write>(&mut self, src: &[u8], writer: &mut W) -> Result<(), std::io::Error> {
        let mut src = src;
        loop {
            let (result, read, _) = self.decoder.decode_to_string(src, &mut self.decoded, false);
            src = &src[read..];
            self.encode_decoded(writer)?;
            match result {
                CoderResult::InputEmpty => return Ok(()),
                CoderResult::OutputFull => {
                    // `decoded` was just drained by encode_decoded; make sure
                    // the retry has head room.
                    self.decoded.reserve(src.len() * 3 + 4);
                }
            }
        }
    }

    /// Flush decoder/encoder end states (pending partial sequences become
    /// U+FFFD, exactly like decoding the whole input with `last = true`).
    fn finish<W: Write>(&mut self, writer: &mut W) -> Result<(), std::io::Error> {
        loop {
            let (result, _, _) = self.decoder.decode_to_string(b"", &mut self.decoded, true);
            self.encode_decoded(writer)?;
            match result {
                CoderResult::InputEmpty => break,
                CoderResult::OutputFull => self.decoded.reserve(64),
            }
        }
        loop {
            let (result, _, _) = self
                .encoder
                .encode_from_utf8_to_vec("", &mut self.out_buf, true);
            write_out(&mut self.out_buf, writer)?;
            match result {
                CoderResult::InputEmpty => break,
                CoderResult::OutputFull => {}
            }
        }
        Ok(())
    }

    /// Encode everything decoded so far and write it out.
    fn encode_decoded<W: Write>(&mut self, writer: &mut W) -> Result<(), std::io::Error> {
        if self.decoded.is_empty() {
            return Ok(());
        }
        let mut consumed = 0usize;
        loop {
            let (result, read, _) = self.encoder.encode_from_utf8_to_vec(
                &self.decoded[consumed..],
                &mut self.out_buf,
                false,
            );
            consumed += read;
            write_out(&mut self.out_buf, writer)?;
            match result {
                CoderResult::InputEmpty => break,
                CoderResult::OutputFull => {}
            }
        }
        self.decoded.clear();
        Ok(())
    }
}

/// Write the converted bytes out and reset the staging buffer. A free
/// function so the encoder loop can borrow `decoded` and `out_buf`
/// independently.
fn write_out<W: Write>(buf: &mut Vec<u8>, writer: &mut W) -> Result<(), std::io::Error> {
    if !buf.is_empty() {
        writer.write_all(buf)?;
        buf.clear();
    }
    Ok(())
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
