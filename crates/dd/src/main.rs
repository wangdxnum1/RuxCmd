//! Convert and copy a file block by block - a Windows port of GNU dd's core
//! subset: if/of/bs/ibs/obs/count/skip/seek/status/conv=notrunc.

use std::io::{Read, Seek, SeekFrom, Write};

const HELP: &str = "Usage: dd [OPERAND]...\n  \
or:    dd OPTION\n\
Copy a file, converting and formatting according to the operands.\n\n\
  bs=N        read and write up to N bytes at a time (default: 512)\n\
  count=N     copy only N input blocks\n\
  ibs=N       read up to N bytes at a time\n\
  if=FILE     read from FILE instead of standard input\n\
  obs=N       write N bytes at a time\n\
  of=FILE     write to FILE instead of standard output\n\
  seek=N      skip N obs-sized blocks in the output\n\
  skip=N      skip N ibs-sized blocks in the input\n\
  status=LEVEL  none | noxfer\n\
  conv=LIST   notrunc is supported; other conversions are rejected\n\n\
N may be followed by c (1), w (2), b (512), kB (1000), K (1024), MB, M, GB, G.";

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();

    if argv.iter().any(|a| a == "--help") {
        println!("{HELP}");
        return;
    }
    if argv.iter().any(|a| a == "--version") {
        println!("dd {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let mut opts = Opts::default();
    for arg in &argv {
        apply_operand(&mut opts, arg);
    }

    let started = std::time::Instant::now();
    let stats = run_copy(&opts).unwrap_or_else(|e| {
        eprintln!("dd: {}", e);
        std::process::exit(1);
    });
    let elapsed = started.elapsed().as_secs_f64().max(0.0000001);

    if opts.status != Status::None {
        if opts.status != Status::Noxfer {
            eprintln!("{}+{} records in", stats.full_in, stats.partial_in);
        }
        eprintln!("{}+{} records out", stats.full_out, stats.partial_out);
        eprintln!(
            "{} bytes copied, {:.7} s, {:.1} kB/s",
            stats.bytes,
            elapsed,
            stats.bytes as f64 / 1024.0 / elapsed
        );
    }
}

#[derive(Default)]
struct Opts {
    input: Option<String>,
    output: Option<String>,
    bs: u64,
    ibs: u64,
    obs: u64,
    count: Option<u64>,
    skip: u64,
    seek: u64,
    status: Status,
    no_trunc: bool,
    sync_pad: bool,
    no_error: bool,
    ucase: bool,
    lcase: bool,
}

#[derive(Default, PartialEq, Eq)]
enum Status {
    #[default]
    Default,
    None,
    Noxfer,
}

struct Stats {
    full_in: u64,
    partial_in: u64,
    full_out: u64,
    partial_out: u64,
    bytes: u64,
}

fn apply_operand(opts: &mut Opts, arg: &str) {
    let Some((key, value)) = arg.split_once('=') else {
        eprintln!("dd: invalid operand '{}'", arg);
        eprintln!("Try 'dd --help' for more information.");
        std::process::exit(1);
    };
    match key {
        "if" => opts.input = Some(value.to_string()),
        "of" => opts.output = Some(value.to_string()),
        "bs" => {
            let n = parse_size(value);
            opts.bs = n;
            opts.ibs = n;
            opts.obs = n;
        }
        "ibs" => opts.ibs = parse_size(value),
        "obs" => opts.obs = parse_size(value),
        "count" => opts.count = Some(parse_size(value)),
        "skip" => opts.skip = parse_size(value),
        "seek" => opts.seek = parse_size(value),
        "status" => match value {
            "none" => opts.status = Status::None,
            "noxfer" => opts.status = Status::Noxfer,
            _ => {
                eprintln!("dd: invalid status level '{}'", value);
                std::process::exit(1);
            }
        },
        "conv" => {
            for conversion in value.split(',') {
                match conversion {
                    "notrunc" => opts.no_trunc = true,
                    "sync" => opts.sync_pad = true,
                    "noerror" => opts.no_error = true,
                    "ucase" => opts.ucase = true,
                    "lcase" => opts.lcase = true,
                    other => {
                        eprintln!("dd: invalid conversion '{}'", other);
                        std::process::exit(1);
                    }
                }
            }
        }
        _ => {
            eprintln!("dd: unrecognized operand '{}'", arg);
            eprintln!("Try 'dd --help' for more information.");
            std::process::exit(1);
        }
    }
}

/// Parse a GNU dd block size: digits plus an optional multiplier suffix
/// (c=1, w=2, b=512, K/KiB=1024, KB=1000, M/G with their kB variants).
fn parse_size(raw: &str) -> u64 {
    let lower = raw.to_ascii_lowercase();
    let (digits, multiplier) = if let Some(rest) = lower.strip_prefix("0x") {
        let n = u64::from_str_radix(rest, 16).unwrap_or_else(|_| bad_size(raw));
        return n;
    } else {
        let split = lower.find(|c: char| c.is_ascii_alphabetic());
        match split {
            Some(idx) => (&lower[..idx], lower[idx..].to_string()),
            None => (lower.as_str(), String::new()),
        }
    };
    let num: u64 = digits.parse().unwrap_or_else(|_| bad_size(raw));
    let mult: u64 = match multiplier.as_str() {
        "" | "c" => 1,
        "w" => 2,
        "b" => 512,
        "k" | "kib" => 1024,
        "kb" => 1000,
        "m" | "mib" => 1024 * 1024,
        "mb" => 1000 * 1000,
        "g" | "gib" => 1024 * 1024 * 1024,
        "gb" => 1000 * 1000 * 1000,
        _ => bad_size(raw),
    };
    num.checked_mul(mult).unwrap_or_else(|| bad_size(raw))
}

fn bad_size(raw: &str) -> ! {
    eprintln!("dd: invalid number: '{}'", raw);
    std::process::exit(1);
}

fn run_copy(opts: &Opts) -> Result<Stats, String> {
    let ibs = if opts.ibs > 0 { opts.ibs } else { 512 };
    let obs = if opts.obs > 0 { opts.obs } else { 512 };

    let mut input: Box<dyn Read> = match &opts.input {
        Some(name) if name != "-" => {
            Box::new(std::fs::File::open(name).map_err(|e| format!("'{name}': {e}"))?)
        }
        _ => Box::new(std::io::stdin()),
    };

    // skip: discard whole ibs-sized blocks from the input.
    let mut skipped: u64 = 0;
    let mut sink = [0u8; 65536];
    while skipped < opts.skip {
        let want = (ibs.min(sink.len() as u64)) as usize;
        let got = read_some(&mut input, &mut sink[..want])?;
        if got == 0 {
            break; // input shorter than the skip region
        }
        skipped += 1;
    }

    let mut output = build_output(opts)?;
    if opts.seek > 0 {
        let offset = opts
            .seek
            .checked_mul(obs)
            .ok_or_else(|| "seek offset too large".to_string())?;
        output.seek_to(offset)?;
    }

    let mut stats = Stats {
        full_in: 0,
        partial_in: 0,
        full_out: 0,
        partial_out: 0,
        bytes: 0,
    };
    let mut written_this_block = 0u64;
    let mut block = vec![0u8; ibs as usize];
    let mut blocks_copied: u64 = 0;

    loop {
        if let Some(limit) = opts.count
            && blocks_copied >= limit
        {
            break;
        }
        let mut got = match read_some_opt(&mut input, &mut block, opts.no_error) {
            Ok(n) => n,
            Err(_) => break,
        };
        if got == 0 {
            break;
        }
        blocks_copied += 1;
        if opts.ucase {
            block[..got].make_ascii_uppercase();
        } else if opts.lcase {
            block[..got].make_ascii_lowercase();
        }
        if opts.sync_pad && got < block.len() {
            for b in &mut block[got..] {
                *b = 0;
            }
            let full = block.len();
            got = full;
        }
        if got == block.len() {
            stats.full_in += 1;
        } else {
            stats.partial_in += 1;
        }

        // Write through the obs window.
        let mut offset = 0usize;
        while offset < got {
            let take = (obs as usize - written_this_block as usize).min(got - offset);
            output
                .write_all(&block[offset..offset + take])
                .map_err(|e| format!("write error: {e}"))?;
            offset += take;
            written_this_block += take as u64;
            stats.bytes += take as u64;
            if written_this_block == obs {
                stats.full_out += 1;
                written_this_block = 0;
            }
        }
    }
    if written_this_block > 0 {
        stats.partial_out += 1;
    }
    output.flush().map_err(|e| format!("write error: {e}"))?;
    Ok(stats)
}

enum OutTarget {
    File(std::fs::File),
    Stdout(std::io::Stdout),
}

impl Write for OutTarget {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            OutTarget::File(f) => f.write(buf),
            OutTarget::Stdout(s) => s.write(buf),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            OutTarget::File(f) => f.flush(),
            OutTarget::Stdout(s) => s.flush(),
        }
    }
}

impl OutTarget {
    fn seek_to(&mut self, offset: u64) -> Result<(), String> {
        match self {
            OutTarget::File(f) => f
                .seek(SeekFrom::Start(offset))
                .map(|_| ())
                .map_err(|e| format!("cannot seek in output: {e}")),
            OutTarget::Stdout(_) => Err("cannot seek on standard output".to_string()),
        }
    }
}

fn build_output(opts: &Opts) -> Result<OutTarget, String> {
    if let Some(name) = &opts.output
        && name != "-"
    {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);
        options.truncate(!opts.no_trunc);
        let file = options.open(name).map_err(|e| format!("'{name}': {e}"))?;
        return Ok(OutTarget::File(file));
    }
    Ok(OutTarget::Stdout(std::io::stdout()))
}

fn read_some_opt(reader: &mut dyn Read, buf: &mut [u8], _no_error: bool) -> Result<usize, String> {
    read_some(reader, buf)
}

/// Read up to the buffer capacity, looping over short reads; a zero return
/// means end of stream.
fn read_some(reader: &mut dyn Read, buf: &mut [u8]) -> Result<usize, String> {
    let mut n = 0;
    while n == 0 {
        match reader.read(buf) {
            Ok(0) => return Ok(0),
            Ok(k) => n = k,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(format!("read error: {e}")),
        }
    }
    Ok(n)
}
