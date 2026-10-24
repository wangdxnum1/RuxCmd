mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encoding reads the input in chunks of this many bytes. It is a multiple
/// of 3, so every full chunk maps onto whole base64 groups without padding.
const ENCODE_CHUNK: usize = 3072;

/// Decoding reads the input in 4-byte-aligned chunks; the bit accumulator
/// carries any remainder across chunk boundaries.
const DECODE_CHUNK: usize = 4096;

/// How a streaming stage failed. Every variant exits 1 after the bytes
/// already produced have been flushed to stdout.
enum Fail {
    /// Print this message to stderr.
    Msg(String),
    /// The decoded input was not valid base64.
    Invalid,
}

/// Error inside the encode/decode pumps, kept separate so the caller can
/// build the same read/write messages as the fully-buffered version did.
enum StageError {
    Read(std::io::Error),
    Write(std::io::Error),
    /// An illegal byte was seen while decoding; stop reading further input.
    Invalid,
}

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("base64 0.1.0");
        return;
    }

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    let result = if args.decode {
        decode_inputs(&args.files, &mut out)
    } else {
        encode_inputs(&args.files, &mut out, args.wrap)
    };

    let mut exit_code = 0i32;
    match result {
        Ok(()) => {
            if let Err(e) = out.flush() {
                eprintln!("base64: write error: {}", e);
                exit_code = 1;
            }
        }
        Err(Fail::Invalid) => {
            // Flush everything decoded before the bad byte first, matching
            // GNU coreutils, which reports the error after the partial output.
            let _ = out.flush();
            eprintln!("base64: invalid input");
            exit_code = 1;
        }
        Err(Fail::Msg(message)) => {
            let _ = out.flush();
            eprintln!("{}", message);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

/// One input operand: stdin ("-" or no files) or a named file.
enum Input<'a> {
    Stdin,
    File(&'a Path),
}

/// Turn a stage failure into the process failure, wording read errors with
/// `source` ("error reading stdin" or the file path) as before.
fn stage_fail(e: StageError, source: &str) -> Fail {
    match e {
        StageError::Read(err) => Fail::Msg(format!("base64: {}: {}", source, err)),
        StageError::Write(err) => Fail::Msg(format!("base64: write error: {}", err)),
        StageError::Invalid => Fail::Invalid,
    }
}

/// Fill `buf` as far as possible, looping over short reads; the byte count
/// returned is < buf.len() only at end of input.
fn read_full<R: Read>(reader: &mut R, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut filled = 0usize;
    while filled < buf.len() {
        let n = reader.read(&mut buf[filled..])?;
        if n == 0 {
            break;
        }
        filled += n;
    }
    Ok(filled)
}

// ---------------------------------------------------------------------------
// Encoding
// ---------------------------------------------------------------------------

/// Stream every input operand through the encoder. The encoder state (line
/// column plus a 0-2 byte carry) spans operands, so multiple files behave
/// exactly like one concatenated input.
fn encode_inputs<W: Write>(files: &[PathBuf], out: &mut W, wrap: usize) -> Result<(), Fail> {
    let mut encoder = Encoder::new(wrap);

    let inputs: Vec<Input> = if files.is_empty() {
        vec![Input::Stdin]
    } else {
        files
            .iter()
            .map(|p| {
                if p.as_os_str() == "-" {
                    Input::Stdin
                } else {
                    Input::File(p)
                }
            })
            .collect()
    };

    for input in &inputs {
        match input {
            Input::Stdin => {
                let stdin = std::io::stdin();
                let mut lock = stdin.lock();
                pump_encode(&mut lock, &mut encoder, out)
                    .map_err(|e| stage_fail(e, "error reading stdin"))?;
            }
            Input::File(path) => {
                let mut file = match File::open(path) {
                    Ok(f) => f,
                    Err(e) => return Err(Fail::Msg(format!("base64: {}: {}", path.display(), e))),
                };
                pump_encode(&mut file, &mut encoder, out)
                    .map_err(|e| stage_fail(e, &path.display().to_string()))?;
            }
        }
    }

    encoder.finish(out).map_err(|e| match e {
        StageError::Write(err) => Fail::Msg(format!("base64: write error: {}", err)),
        StageError::Read(_) | StageError::Invalid => {
            unreachable!("finish neither reads nor decodes")
        }
    })
}

fn pump_encode<R: Read, W: Write>(
    reader: &mut R,
    encoder: &mut Encoder,
    out: &mut W,
) -> Result<(), StageError> {
    let mut buf = [0u8; ENCODE_CHUNK];
    loop {
        let filled = read_full(reader, &mut buf).map_err(StageError::Read)?;
        if filled == 0 {
            return Ok(());
        }
        encoder.feed(&buf[..filled], out)?;
    }
}

/// Streaming base64 encoder: emits complete groups per chunk and carries an
/// incomplete 0-2 byte tail into the next chunk (or into `finish`).
struct Encoder {
    wrap: usize,
    col: usize,
    carry: [u8; 2],
    carry_len: usize,
}

impl Encoder {
    fn new(wrap: usize) -> Self {
        Encoder {
            wrap,
            col: 0,
            carry: [0u8; 2],
            carry_len: 0,
        }
    }

    fn feed<W: Write>(&mut self, data: &[u8], out: &mut W) -> Result<(), StageError> {
        let mut rest = data;
        // Complete a group left over from the previous chunk first.
        if self.carry_len > 0 {
            let take = (3 - self.carry_len).min(rest.len());
            // The completed group is built in a local: the persistent carry
            // only ever holds 0-2 bytes.
            let mut group = [0u8; 3];
            group[..self.carry_len].copy_from_slice(&self.carry[..self.carry_len]);
            group[self.carry_len..self.carry_len + take].copy_from_slice(&rest[..take]);
            self.carry_len += take;
            rest = &rest[take..];
            if self.carry_len == 3 {
                self.emit(&group, 3, out)?;
                self.carry_len = 0;
            }
        }
        let whole = rest.len() - rest.len() % 3;
        for group in rest[..whole].as_chunks::<3>().0 {
            self.emit(group, 3, out)?;
        }
        let tail = &rest[whole..];
        self.carry[..tail.len()].copy_from_slice(tail);
        self.carry_len = tail.len();
        Ok(())
    }

    /// Encode the carried tail with '=' padding and emit the final newline.
    fn finish<W: Write>(&mut self, out: &mut W) -> Result<(), StageError> {
        if self.carry_len > 0 {
            let mut group = [0u8; 3];
            group[..self.carry_len].copy_from_slice(&self.carry[..self.carry_len]);
            self.emit(&group, self.carry_len, out)?;
            self.carry_len = 0;
        }
        if self.wrap > 0 && self.col > 0 {
            out.write_all(b"\n").map_err(StageError::Write)?;
        }
        Ok(())
    }

    /// Emit one group; `n` is the number of meaningful bytes (1-3), the
    /// remainder is zero-padded and rendered as '='.
    fn emit<W: Write>(&mut self, b: &[u8], n: usize, out: &mut W) -> Result<(), StageError> {
        let idx = [
            (b[0] >> 2) as usize,
            (((b[0] & 0x03) << 4) | (b[1] >> 4)) as usize,
            (((b[1] & 0x0F) << 2) | (b[2] >> 6)) as usize,
            (b[2] & 0x3F) as usize,
        ];
        let mut chars = [0u8; 4];
        chars[0] = ALPHABET[idx[0]];
        chars[1] = ALPHABET[idx[1]];
        chars[2] = if n >= 2 { ALPHABET[idx[2]] } else { b'=' };
        chars[3] = if n >= 3 { ALPHABET[idx[3]] } else { b'=' };
        out.write_all(&chars).map_err(StageError::Write)?;
        self.col += 4;
        if self.wrap > 0 && self.col >= self.wrap {
            out.write_all(b"\n").map_err(StageError::Write)?;
            self.col = 0;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Decoding
// ---------------------------------------------------------------------------

/// Stream every input operand through the decoder. Reading stops at the
/// first illegal byte; `finish` then emits the partial group and reports
/// the failure, exactly like the buffered version did.
fn decode_inputs<W: Write>(files: &[PathBuf], out: &mut W) -> Result<(), Fail> {
    let mut decoder = Decoder::new();

    let inputs: Vec<Input> = if files.is_empty() {
        vec![Input::Stdin]
    } else {
        files
            .iter()
            .map(|p| {
                if p.as_os_str() == "-" {
                    Input::Stdin
                } else {
                    Input::File(p)
                }
            })
            .collect()
    };

    for input in &inputs {
        let stage = match input {
            Input::Stdin => {
                let stdin = std::io::stdin();
                let mut lock = stdin.lock();
                pump_decode(&mut lock, &mut decoder, out)
                    .map_err(|e| stage_fail(e, "error reading stdin"))
            }
            Input::File(path) => {
                let mut file = match File::open(path) {
                    Ok(f) => f,
                    Err(e) => return Err(Fail::Msg(format!("base64: {}: {}", path.display(), e))),
                };
                pump_decode(&mut file, &mut decoder, out)
                    .map_err(|e| stage_fail(e, &path.display().to_string()))
            }
        };
        match stage {
            Ok(()) => {}
            // Stop reading further operands; finish reports the failure
            // after emitting what was already decoded.
            Err(Fail::Invalid) => break,
            Err(other) => return Err(other),
        }
    }

    match decoder.finish(out) {
        Ok(()) => Ok(()),
        Err(StageError::Write(err)) => Err(Fail::Msg(format!("base64: write error: {}", err))),
        Err(StageError::Invalid) => Err(Fail::Invalid),
        Err(StageError::Read(_)) => unreachable!("finish does not read"),
    }
}

fn pump_decode<R: Read, W: Write>(
    reader: &mut R,
    decoder: &mut Decoder,
    out: &mut W,
) -> Result<(), StageError> {
    let mut buf = [0u8; DECODE_CHUNK];
    loop {
        let filled = read_full(reader, &mut buf).map_err(StageError::Read)?;
        if filled == 0 {
            return Ok(());
        }
        decoder.feed(&buf[..filled], out)?;
    }
}

/// Streaming base64 decoder. The accumulator and padding latch carry across
/// chunks, so chunk boundaries are invisible to the decoding state machine.
struct Decoder {
    table: [u8; 256],
    acc: u32,
    bits: u32,
    pad_seen: bool,
    valid: bool,
}

impl Decoder {
    fn new() -> Self {
        let mut table = [255u8; 256];
        for (i, &c) in ALPHABET.iter().enumerate() {
            table[c as usize] = i as u8;
        }
        Decoder {
            table,
            acc: 0,
            bits: 0,
            pad_seen: false,
            valid: true,
        }
    }

    /// Feed ciphertext bytes. Returns `StageError::Invalid` at the first
    /// illegal byte; the partially accumulated group is kept for `finish`.
    fn feed<W: Write>(&mut self, data: &[u8], out: &mut W) -> Result<(), StageError> {
        for &ch in data {
            match ch {
                b'\n' | b'\r' | b' ' | b'\t' => {}
                b'=' => self.pad_seen = true,
                _ => {
                    let v = self.table[ch as usize];
                    if v == 255 || self.pad_seen {
                        self.valid = false;
                        return Err(StageError::Invalid);
                    }
                    self.acc = (self.acc << 6) | (v as u32);
                    self.bits += 6;
                    if self.bits >= 24 {
                        let out3 = [
                            ((self.acc >> 16) & 0xFF) as u8,
                            ((self.acc >> 8) & 0xFF) as u8,
                            (self.acc & 0xFF) as u8,
                        ];
                        out.write_all(&out3).map_err(StageError::Write)?;
                        self.acc = 0;
                        self.bits = 0;
                    }
                }
            }
        }
        Ok(())
    }

    /// Emit whatever whole bytes remain in the accumulator (the bits past
    /// bits/8 are the zero padding of the final sextet group) and report
    /// whether the input seen so far was valid base64.
    fn finish<W: Write>(&mut self, out: &mut W) -> Result<(), StageError> {
        let whole = self.bits / 8;
        for i in 0..whole {
            let shift = self.bits - 8 * (i + 1);
            out.write_all(&[((self.acc >> shift) & 0xFF) as u8])
                .map_err(StageError::Write)?;
        }
        if !self.valid || (self.bits > 0 && !self.pad_seen) {
            return Err(StageError::Invalid);
        }
        Ok(())
    }
}
