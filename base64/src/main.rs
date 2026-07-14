mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

const ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("base64 0.1.0");
        return;
    }

    let mut exit_code = 0i32;

    let mut data: Vec<u8> = Vec::new();

    let result = if args.files.is_empty() {
        read_all_stdin(&mut data)
    } else {
        let mut ok = true;
        for p in &args.files {
            let r = if p.as_os_str() == "-" {
                read_all_stdin(&mut data)
            } else {
                read_all_file(p, &mut data)
            };
            if r.is_err() {
                ok = false;
                exit_code = 1;
            }
        }
        if ok { Ok(()) } else { Err(()) }
    };
    if result.is_err() {
        std::process::exit(exit_code);
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let res = if args.decode {
        decode_write(&data, &mut out)
    } else {
        encode_write(&data, &mut out, args.wrap)
    };
    if let Err(e) = res {
        eprintln!("base64: write error: {}", e);
        exit_code = 1;
    }

    std::process::exit(exit_code);
}

fn read_all_stdin(buf: &mut Vec<u8>) -> Result<(), ()> {
    let stdin = std::io::stdin();
    if let Err(e) = stdin.lock().read_to_end(buf) {
        eprintln!("base64: error reading stdin: {}", e);
        return Err(());
    }
    Ok(())
}

fn read_all_file(p: &std::path::Path, buf: &mut Vec<u8>) -> Result<(), ()> {
    match File::open(p) {
        Ok(mut f) => {
            if let Err(e) = f.read_to_end(buf) {
                eprintln!("base64: {}: {}", p.display(), e);
                return Err(());
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("base64: {}: {}", p.display(), e);
            Err(())
        }
    }
}

fn encode_write<W: Write>(input: &[u8], out: &mut W, wrap: usize) -> Result<(), std::io::Error> {
    let mut col = 0usize;
    let mut i = 0usize;
    while i < input.len() {
        let n = std::cmp::min(3, input.len() - i);
        let mut b = [0u8; 3];
        b[..n].copy_from_slice(&input[i..i + n]);
        let idx = [
            (b[0] >> 2) as usize,
            (((b[0] & 0x03) << 4) | (b[1] >> 4)) as usize,
            (((b[1] & 0x0F) << 2) | (b[2] >> 6)) as usize,
            (b[2] & 0x3F) as usize,
        ];
        let mut chars = [0u8; 4];
        chars[0] = ALPHABET[idx[0]];
        chars[1] = ALPHABET[idx[1]];
        if n >= 2 {
            chars[2] = ALPHABET[idx[2]];
        } else {
            chars[2] = b'=';
        }
        if n >= 3 {
            chars[3] = ALPHABET[idx[3]];
        } else {
            chars[3] = b'=';
        }
        out.write_all(&chars)?;
        col += 4;
        if wrap > 0 && col >= wrap {
            out.write_all(b"\n")?;
            col = 0;
        }
        i += n;
    }
    if wrap > 0 && col > 0 {
        out.write_all(b"\n")?;
    }
    Ok(())
}

fn decode_write<W: Write>(input: &[u8], out: &mut W) -> Result<(), std::io::Error> {
    let mut table = [255u8; 256];
    for (i, &c) in ALPHABET.iter().enumerate() {
        table[c as usize] = i as u8;
    }
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    let mut pad = 0u32;
    for &ch in input {
        match ch {
            b'\n' | b'\r' | b' ' | b'\t' => continue,
            b'=' => {
                pad += 1;
                bits += 6;
                if bits >= 24 {
                    let mut out3 = [0u8; 3];
                    out3[0] = ((acc >> 16) & 0xFF) as u8;
                    out3[1] = ((acc >> 8) & 0xFF) as u8;
                    out3[2] = (acc & 0xFF) as u8;
                    let write_n = match pad {
                        1 => 2,
                        2 => 1,
                        _ => 0,
                    };
                    if write_n > 0 {
                        out.write_all(&out3[..write_n])?;
                    }
                    acc = 0; bits = 0; pad = 0;
                }
            }
            _ => {
                let v = table[ch as usize];
                if v == 255 {
                    eprintln!("base64: invalid character in input");
                    std::process::exit(1);
                }
                acc = (acc << 6) | (v as u32);
                bits += 6;
                if bits >= 24 {
                    let out3 = [
                        ((acc >> 16) & 0xFF) as u8,
                        ((acc >> 8) & 0xFF) as u8,
                        (acc & 0xFF) as u8,
                    ];
                    out.write_all(&out3)?;
                    acc = 0; bits = 0; pad = 0;
                }
            }
        }
    }
    if bits >= 12 {
        let n = if bits >= 24 { 3 } else if bits >= 18 { 2 } else if pad > 0 {
            if pad == 1 { 2 } else { 1 }
        } else { 1 };
        let shifts = [16u32, 8u32, 0u32];
        for i in 0..n {
            let b = ((acc >> shifts[i]) & 0xFF) as u8;
            out.write_all(&[b])?;
        }
    }
    Ok(())
}
