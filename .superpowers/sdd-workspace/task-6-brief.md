# Task 6: csplit + pr + numfmt 三个 Crate

- csplit：按正则表达式/行号把文件分割成 xx00 xx01 ... 多个分片
- pr：文本分页格式化（行号、日期、页眉、页宽、双栏、跳过空页）
- numfmt：大数字人性化格式（--to=auto/si/iec/iec-i 四模式，--from，--field，--padding，--header，--suffix，--round）

9 个新文件：
- csplit/Cargo.toml、csplit/src/cli.rs、csplit/src/main.rs
- pr/Cargo.toml、pr/src/cli.rs、pr/src/main.rs
- numfmt/Cargo.toml、numfmt/src/cli.rs、numfmt/src/main.rs

**PRE-FLIGHT CONFLICT #2 FIX (numfmt iec-i 后缀)：**
原 plan 的 numfmt iec-i 模式下后缀缺少 i（写成 "K"/"M"/...）。正确应为：iec-i 模式在二进制缩放（1024^n）基础上 **追加小写 i**，即 "KiB"/"MiB"/"GiB"/...。Brief Step 3 numfmt main.rs 已写修正版。

Pre-req: 先建 csplit/src、pr/src、numfmt/src 三个目录。

---

## Part A: csplit crate

### Step 1: csplit/Cargo.toml

```toml
[package]
name = "csplit"
version = "0.1.0"
edition = "2024"
description = "Split a file into sections determined by context lines"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

### Step 2: csplit/src/cli.rs

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "csplit", version, about = "Split FILE by PATTERN to xxNN output files", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'f', long = "prefix", default_value = "xx")]
    pub prefix: String,

    #[arg(short = 'b', long = "suffix-format", default_value = "%02d")]
    pub suffix_format: String,

    #[arg(short = 'n', long = "digits", default_value_t = 2usize)]
    pub digits: usize,

    #[arg(short = 'k', long = "keep-files", action = clap::ArgAction::SetTrue)]
    pub keep: bool,

    #[arg(short = 'z', long = "elide-empty-files", action = clap::ArgAction::SetTrue)]
    pub elide_empty: bool,

    #[arg(short = 's', long = "quiet", action = clap::ArgAction::SetTrue)]
    pub quiet: bool,

    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    #[arg(value_name = "PATTERNS", last = true)]
    pub patterns: Vec<String>,
}
```

### Step 3: csplit/src/main.rs

```rust
mod cli;

use clap::Parser;
use std::fs::{File, remove_file};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("csplit 0.1.0");
        return;
    }

    if args.patterns.is_empty() {
        eprintln!("csplit: missing pattern(s)");
        std::process::exit(1);
    }

    let lines: Vec<String> = match read_input(&args.file) {
        Ok(v) => v,
        Err(e) => { eprintln!("csplit: {}", e); std::process::exit(1); }
    };

    let mut splits: Vec<(usize, usize)> = Vec::new();
    let mut cur: usize = 0;
    let mut created: Vec<PathBuf> = Vec::new();
    let mut exit_code = 0i32;

    let mut pat_idx: usize = 0;
    while pat_idx < args.patterns.len() {
        let p = &args.patterns[pat_idx];
        if let Some(repeat) = try_repeat_tail(p) {
            let base = p[..p.len()-1].to_string();
            for _ in 0..repeat {
                let (end, offset) = match next_match(&base, &lines, cur) {
                    Some(v) => v,
                    None => break,
                };
                let start = cur;
                if offset > 0 {
                    splits.push((start, end));
                    cur = end;
                    if end + offset < end || end + offset > lines.len() {
                        break;
                    }
                    cur = end + offset;
                } else {
                    splits.push((start, end + offset));
                    cur = end + offset;
                }
            }
            pat_idx += 1;
        } else if p.starts_with("/") && p.ends_with("/") {
            let pat = &p[1..p.len()-1];
            match find_pattern(pat, &lines, cur) {
                Some(lineno) => {
                    splits.push((cur, lineno));
                    cur = lineno;
                }
                None => {
                    splits.push((cur, lines.len()));
                    cur = lines.len();
                }
            }
            pat_idx += 1;
        } else if let Ok(n) = p.parse::<usize>() {
            if n == 0 || n > lines.len() {
                splits.push((cur, lines.len()));
                cur = lines.len();
            } else {
                splits.push((cur, n - 1));
                cur = n - 1;
            }
            pat_idx += 1;
        } else {
            eprintln!("csplit: invalid pattern '{}'", p);
            exit_code = 1;
            pat_idx += 1;
        }
    }
    if cur < lines.len() {
        splits.push((cur, lines.len()));
    }

    let mut file_count = 0usize;
    for (s, e) in &splits {
        if args.elide_empty && s >= e {
            continue;
        }
        let fname = format_filename(&args.prefix, file_count, args.digits);
        let path = PathBuf::from(&fname);
        match File::create(&path) {
            Ok(mut f) => {
                for i in *s..(*e.min(&lines.len())) {
                    let _ = writeln!(f, "{}", lines[i]);
                }
                let bytes: usize = lines[*s..(*e.min(&lines.len()))].iter()
                    .map(|l| l.len() + 1).sum();
                if !args.quiet { println!("{}", bytes); }
                created.push(path);
                file_count += 1;
            }
            Err(err) => {
                eprintln!("csplit: {}: {}", fname, err);
                exit_code = 1;
            }
        }
    }

    if exit_code != 0 && !args.keep {
        for p in &created { let _ = remove_file(p); }
    }

    std::process::exit(exit_code);
}

fn read_input(p: &Option<PathBuf>) -> Result<Vec<String>, String> {
    let mut reader: Box<dyn BufRead> = match p {
        Some(pp) if pp.to_string_lossy() != "-" => {
            let f = File::open(pp).map_err(|e| format!("{}: {}", pp.display(), e))?;
            Box::new(BufReader::new(f))
        }
        _ => {
            let stdin = std::io::stdin();
            Box::new(BufReader::new(stdin.lock()))
        }
    };
    let mut out = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(l) => out.push(l),
            Err(e) => return Err(format!("read error: {}", e)),
        }
    }
    Ok(out)
}

fn try_repeat_tail(p: &str) -> Option<usize> {
    if let Some(idx) = p.rfind(|c: char| !c.is_ascii_digit()) {
        if idx + 1 < p.len() {
            let n: usize = p[idx+1..].parse().ok()?;
            if n > 0 { Some(n) } else { None }
        } else { None }
    } else { None }
}

fn next_match(pat: &str, lines: &[String], cur: usize) -> Option<(usize, usize)> {
    if pat.starts_with("/") && pat.ends_with("/") {
        let inner = &pat[1..pat.len()-1];
        find_pattern(inner, lines, cur).map(|ln| (ln, 1))
    } else if let Ok(n) = pat.parse::<usize>() {
        if n < cur + 1 { None } else { Some((n - 1, 1)) }
    } else { None }
}

fn find_pattern(pat: &str, lines: &[String], start: usize) -> Option<usize> {
    for (i, l) in lines.iter().enumerate().skip(start) {
        if l.contains(pat) {
            return Some(i);
        }
    }
    None
}

fn format_filename(prefix: &str, idx: usize, digits: usize) -> String {
    format!("{}{:0w$}", prefix, idx, w = digits)
}
```

### Step 4: csplit 构建

```
cd /d d:\Work\rust\csplit && cargo build --release --offline
```

---

## Part B: pr crate

### Step 5: pr/Cargo.toml

```toml
[package]
name = "pr"
version = "0.1.0"
edition = "2024"
description = "Paginate or columnate files for printing"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

### Step 6: pr/src/cli.rs

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "pr", version, about = "Paginate or columnate FILE(s) for printing", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'd', long = "double-space", action = clap::ArgAction::SetTrue)]
    pub double_space: bool,

    #[arg(short = 'l', long = "length", default_value_t = 66usize)]
    pub length: usize,

    #[arg(short = 'o', long = "indent", default_value_t = 0usize)]
    pub indent: usize,

    #[arg(short = 'W', long = "page-width", default_value_t = 72usize)]
    pub page_width: usize,

    #[arg(short = 'h', long = "header", value_name = "HEADER")]
    pub header: Option<String>,

    #[arg(short = 'f', long = "form-feed", action = clap::ArgAction::SetTrue)]
    pub form_feed: bool,

    #[arg(short = 'n', long = "number-lines", value_name = "SEP[N]", default_value = None)]
    pub number_lines: Option<String>,

    #[arg(short = 't', long = "omit-header", action = clap::ArgAction::SetTrue)]
    pub omit_header: bool,

    #[arg(short = '2', long = "two-column", action = clap::ArgAction::SetTrue)]
    pub two_column: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
```

### Step 7: pr/src/main.rs

```rust
mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("pr 0.1.0");
        return;
    }

    let default_header = match args.file.as_ref().and_then(|_| args.files.first()).cloned() {
        Some(p) => p.display().to_string(),
        None => String::from(""),
    };
    let header_text = args.header.clone().unwrap_or(default_header);

    let date_str = format_date();

    let mut all_lines: Vec<String> = Vec::new();
    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = read_lines(&mut BufReader::new(stdin.lock()), &mut all_lines) {
            eprintln!("pr: stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for p in &args.files {
            if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                if let Err(e) = read_lines(&mut BufReader::new(stdin.lock()), &mut all_lines) {
                    eprintln!("pr: stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(p) {
                    Ok(f) => if let Err(e) = read_lines(&mut BufReader::new(f), &mut all_lines) {
                        eprintln!("pr: {}: {}", p.display(), e);
                        exit_code = 1;
                    },
                    Err(e) => { eprintln!("pr: {}: {}", p.display(), e); exit_code = 1; }
                }
            }
        }
    }

    let (num_sep, num_width) = parse_number_spec(&args.number_lines);
    let indent_str: String = " ".repeat(args.indent);
    let page_len = if args.length <= 10 { 66 } else { args.length };
    let header_lines = if args.omit_header { 0 } else { 5 };
    let body_len = page_len - header_lines;

    let processed: Vec<String> = preprocess(&all_lines, &args);
    let page_width = args.page_width.max(1);

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let pages = if processed.is_empty() { 1 } else {
        (processed.len() + body_len - 1) / body_len.max(1)
    };

    for pno in 0..pages {
        let start = pno * body_len;
        let end = std::cmp::min(start + body_len, processed.len());
        let body: &[String] = if start < end { &processed[start..end] } else { &[] };

        if !args.omit_header {
            let _ = write_header(&mut out, &indent_str, &header_text, &date_str, pno + 1, page_width);
        }

        for (i, line) in body.iter().enumerate() {
            let mut out_line = String::new();
            out_line.push_str(&indent_str);
            if let Some(sep) = num_sep.as_ref() {
                let num = start + i + 1;
                out_line.push_str(&format!("{:>w$}{}", num, sep, w = num_width));
            }
            let padded = truncate_or_keep(line, page_width - (out_line.len() - indent_str.len()));
            out_line.push_str(&padded);
            if args.double_space {
                out_line.push('\n');
            }
            let _ = writeln!(out, "{}", out_line);
        }

        let trailer_len = body_len - body.len();
        if !args.omit_header && !args.form_feed {
            for _ in 0..trailer_len {
                let _ = writeln!(out, "{}", indent_str);
            }
        }

        if args.form_feed {
            let _ = out.write_all(b"\x0C");
        }
    }

    std::process::exit(exit_code);
}

fn format_date() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let y = 1970 + (secs / 31536000);
    let m = ((secs / 2592000) % 12) + 1;
    let d = ((secs / 86400) % 30) + 1;
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn parse_number_spec(spec: &Option<String>) -> (Option<String>, usize) {
    match spec {
        None => (None, 0),
        Some(s) => {
            let mut split = String::new();
            let mut w: usize = 0;
            let mut found_split = false;
            for c in s.chars() {
                if c.is_ascii_digit() && !found_split {
                    w = w * 10 + (c as usize - '0' as usize);
                } else {
                    found_split = true;
                    split.push(c);
                }
            }
            if w == 0 { w = 5; }
            let sep = if found_split { split } else { String::from("\t") };
            (Some(sep), w)
        }
    }
}

fn read_lines<R: BufRead>(r: &mut R, out: &mut Vec<String>) -> Result<(), std::io::Error> {
    for line in r.lines() {
        out.push(line?);
    }
    Ok(())
}

fn preprocess(lines: &[String], args: &cli::Args) -> Vec<String> {
    if args.two_column {
        let n = lines.len();
        let half = (n + 1) / 2;
        let col_w = (args.page_width / 2).saturating_sub(1);
        let mut out = Vec::with_capacity(half);
        for i in 0..half {
            let mut row = String::new();
            if i < lines.len() {
                let a = truncate_or_keep(&lines[i], col_w);
                row.push_str(&a);
                for _ in a.len()..col_w { row.push(' '); }
            } else {
                for _ in 0..col_w { row.push(' '); }
            }
            row.push(' ');
            let ri = half + i;
            if ri < lines.len() {
                let b = truncate_or_keep(&lines[ri], col_w);
                row.push_str(&b);
            }
            out.push(row);
        }
        out
    } else {
        lines.to_vec()
    }
}

fn truncate_or_keep(s: &str, w: usize) -> String {
    let mut result = String::with_capacity(w);
    let mut count = 0usize;
    for c in s.chars() {
        let cw = if (c as u32) < 0x80 { 1 } else { 2 };
        if count + cw > w { break; }
        result.push(c);
        count += cw;
    }
    result
}

fn write_header<W: Write>(out: &mut W, indent: &str, header: &str, date: &str, page_no: usize, page_w: usize) -> Result<(), std::io::Error> {
    for _ in 0..=page_no.max(1) { if page_no == 1 {} }
    let total_inner = page_w - indent.len();
    let page_text = format!("Page {}", page_no);
    let center_head = center(header, total_inner);
    let mut line_a = String::from(indent);
    line_a.push_str(&truncate_or_keep(date, total_inner.min(date.len())));
    for _ in line_a.len()..page_w { line_a.push(' '); }
    let pg_start = page_w.saturating_sub(page_text.len());
    line_a.truncate(pg_start);
    line_a.push_str(&page_text);
    writeln!(out, "{}", line_a)?;

    let center_line = format!("{}{}", indent, center_head);
    writeln!(out, "{}", center_line)?;

    let mut sep = String::from(indent);
    for _ in 0..total_inner { sep.push('='); }
    writeln!(out, "{}", sep)?;
    writeln!(out, "")?;
    writeln!(out, "")?;
    Ok(())
}

fn center(s: &str, w: usize) -> String {
    let n = s.len();
    if n >= w { return s.to_string(); }
    let lpad = (w - n) / 2;
    let rpad = w - n - lpad;
    format!("{}{}{}", " ".repeat(lpad), s, " ".repeat(rpad))
}
```

### Step 8: pr 构建

```
cd /d d:\Work\rust\pr && cargo build --release --offline
```

---

## Part C: numfmt crate（含 PRE-FLIGHT FIX: iec-i 追加 i 后缀）

### Step 9: numfmt/Cargo.toml

```toml
[package]
name = "numfmt"
version = "0.1.0"
edition = "2024"
description = "Reformat numbers in a human-readable way"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

### Step 10: numfmt/src/cli.rs

```rust
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "numfmt", version, about = "Reformat NUMBER(s), or the numbers from FILE", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(long = "to", value_enum, default_value_t = To::None)]
    pub to: To,

    #[arg(long = "from", value_enum, default_value_t = From::None)]
    pub from: From,

    #[arg(long = "to-unit", default_value_t = 1f64)]
    pub to_unit: f64,

    #[arg(long = "from-unit", default_value_t = 1f64)]
    pub from_unit: f64,

    #[arg(short = 'd', long = "delimiter", default_value = " \t")]
    pub delimiter: String,

    #[arg(long = "field", value_name = "N", default_value_t = 1usize)]
    pub field: usize,

    #[arg(short = 'H', long = "header", default_value_t = 0usize)]
    pub header: usize,

    #[arg(short = 'p', long = "padding", value_name = "N", default_value_t = 0isize)]
    pub padding: isize,

    #[arg(short = 'S', long = "suffix", value_name = "SUF")]
    pub suffix_opt: Option<String>,

    #[arg(long = "round", value_enum, default_value_t = Round::Up)]
    pub round: Round,

    #[arg(long = "grouping", action = clap::ArgAction::SetTrue)]
    pub grouping: bool,

    #[arg(value_name = "NUMBER_OR_FILE")]
    pub rest: Vec<String>,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum To {
    None,
    Auto,
    Si,
    Iec,
    #[value(name = "iec-i")]
    IecI,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum From {
    None,
    Auto,
    Si,
    Iec,
    #[value(name = "iec-i")]
    IecI,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum Round {
    Up,
    Down,
    FromZero,
    TowardsZero,
    Nearest,
}
```

### Step 11: numfmt/src/main.rs（iec-i 追加 i 已在 suffix 中处理）

```rust
mod cli;

use clap::Parser;
use cli::{From, Round, To};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

const SI_SUFFIXES: [char; 8] = [' ', 'K', 'M', 'G', 'T', 'P', 'E', 'Z'];
const IEC_SUFFIXES: [char; 8] = SI_SUFFIXES;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("numfmt 0.1.0");
        return;
    }

    let suffix = args.suffix_opt.clone().unwrap_or_default();
    let is_file = !args.rest.is_empty() && std::path::Path::new(&args.rest[0]).exists();
    let mut exit_code = 0i32;

    let mut process = |line: &str, is_header: bool| {
        if is_header {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            let _ = writeln!(out, "{}", line);
            return;
        }
        let pieces: Vec<&str> = split_fields(line, &args.delimiter);
        let fidx = args.field.checked_sub(1).unwrap_or(0);
        if fidx >= pieces.len() {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            let _ = writeln!(out, "{}", line);
            return;
        }
        let token = pieces[fidx].trim();
        let value = match parse_numeric(token, args.from, args.from_unit, &suffix) {
            Some(v) => v,
            None => {
                eprintln!("numfmt: invalid number '{}'", token);
                exit_code = 1;
                let stdout = std::io::stdout();
                let mut out = stdout.lock();
                let _ = writeln!(out, "{}", line);
                return;
            }
        };
        let formatted = format_numeric(value, args.to, args.to_unit, args.round, &suffix, args.grouping);
        let mut joined = String::new();
        for (i, piece) in pieces.iter().enumerate() {
            if i > 0 { joined.push(' '); }
            if i == fidx {
                joined.push_str(&formatted);
            } else {
                joined.push_str(piece);
            }
        }
        let padded = apply_padding(&joined, args.padding);
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        let _ = writeln!(out, "{}", padded);
    };

    if args.rest.is_empty() || !is_file {
        let numbers: Vec<String> = args.rest.clone();
        if numbers.is_empty() {
            let stdin = std::io::stdin();
            for (i, line) in BufReader::new(stdin.lock()).lines().enumerate() {
                match line {
                    Ok(l) => process(&l, i < args.header),
                    Err(e) => { eprintln!("numfmt: stdin: {}", e); exit_code = 1; }
                }
            }
        } else {
            for (i, n) in numbers.iter().enumerate() {
                process(n, i < args.header);
            }
        }
    } else {
        let fname = &args.rest[0];
        match File::open(fname) {
            Ok(f) => for (i, line) in BufReader::new(f).lines().enumerate() {
                match line {
                    Ok(l) => process(&l, i < args.header),
                    Err(e) => { eprintln!("numfmt: {}: {}", fname, e); exit_code = 1; }
                }
            },
            Err(e) => { eprintln!("numfmt: {}: {}", fname, e); exit_code = 1; }
        }
    }

    std::process::exit(exit_code);
}

fn split_fields<'a>(line: &'a str, delim: &str) -> Vec<&'a str> {
    let ds: Vec<char> = delim.chars().collect();
    let mut out = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut start = 0;
    let mut in_sep = true;
    for (i, c) in chars.iter().enumerate() {
        if ds.contains(c) {
            if !in_sep {
                out.push(slice_from_chars(line, &chars, start, i));
            }
            in_sep = true;
        } else {
            if in_sep { start = i; in_sep = false; }
        }
    }
    if !in_sep {
        out.push(slice_from_chars(line, &chars, start, chars.len()));
    }
    out
}

fn slice_from_chars<'a>(src: &'a str, chars: &[char], s: usize, e: usize) -> &'a str {
    let byte_s = chars[..s].iter().map(|c| c.len_utf8()).sum::<usize>();
    let byte_e = chars[s..e.min(chars.len())].iter().map(|c| c.len_utf8()).sum::<usize>() + byte_s;
    &src[byte_s..byte_e.min(src.len())]
}

fn parse_numeric(s: &str, from: From, from_unit: f64, suffix: &str) -> Option<f64> {
    let s = s.strip_suffix(suffix).unwrap_or(s);
    let last = s.chars().last()?;
    let (mult, rest) = match from {
        From::None => {
            match s.parse::<f64>() {
                Ok(v) => return Some(v * from_unit),
                Err(_) => return None,
            }
        }
        From::Si => parse_suffix(s, false),
        From::Iec => parse_suffix(s, true),
        From::IecI => parse_suffix(s, true),
    };
    let v: f64 = rest.parse().ok()?;
    Some(v * mult * from_unit)
}

fn parse_suffix(s: &str, binary: bool) -> (f64, &str) {
    let base: f64 = if binary { 1024.0 } else { 1000.0 };
    let last = match s.chars().last() {
        Some(c) => c,
        None => return (1.0, s),
    };
    if last.is_ascii_digit() || last == '.' || last == '-' {
        return (1.0, s);
    }
    let i = match last.to_ascii_uppercase() {
        'K' => 1, 'M' => 2, 'G' => 3, 'T' => 4, 'P' => 5, 'E' => 6, 'Z' => 7,
        _ => return (1.0, s),
    };
    let trimmed = &s[..s.len()-1];
    (base.powi(i as i32), trimmed)
}

fn format_numeric(v: f64, to: To, to_unit: f64, rnd: Round, suffix: &str, grouping: bool) -> String {
    let value = v / to_unit;
    let base: f64 = match to {
        To::Si => 1000.0,
        To::Iec | To::IecI => 1024.0,
        To::Auto => if value >= 1024.0 { 1024.0 } else { 1000.0 },
        To::None => { return format_plain(value, grouping, 0, rnd, suffix); }
    };
    let mut idx = 0usize;
    let mut n = value;
    while idx + 1 < SI_SUFFIXES.len() && n.abs() >= base - 1e-9 {
        n /= base;
        idx += 1;
    }
    let s = if idx == 0 { String::new() } else {
        let ch = if matches!(to, To::Si) { SI_SUFFIXES[idx] } else { IEC_SUFFIXES[idx] };
        // PRE-FLIGHT FIX: for iec-i, append lowercase "i" after suffix letter (Ki Mi Gi Ti...)
        let ieci = if matches!(to, To::IecI) { "i" } else { "" };
        format!("{}{}", ch, ieci)
    };
    let precision = if idx == 0 { 0 } else { 1 };
    let num_str = format_plain(n, grouping, precision, rnd, "");
    format!("{}{}{}", num_str, s, suffix)
}

fn format_plain(v: f64, grouping: bool, decimals: usize, rnd: Round, suffix: &str) -> String {
    let rounded = apply_round(v, rnd, decimals);
    let mut out = if decimals == 0 {
        format!("{:.0}", rounded)
    } else {
        format!("{:.*}", decimals, rounded)
    };
    if grouping {
        out = add_grouping(&out);
    }
    out.push_str(suffix);
    out
}

fn apply_round(v: f64, r: Round, decimals: usize) -> f64 {
    let fact = 10f64.powi(decimals as i32);
    let scaled = v * fact;
    let done = match r {
        Round::Up => scaled.ceil(),
        Round::Down => scaled.floor(),
        Round::FromZero => if scaled >= 0.0 { scaled.ceil() } else { scaled.floor() },
        Round::TowardsZero => scaled.trunc(),
        Round::Nearest => scaled.round(),
    };
    done / fact
}

fn add_grouping(s: &str) -> String {
    let neg = s.starts_with('-');
    let body = if neg { &s[1..] } else { s };
    let mut dot_idx = body.find('.').unwrap_or(body.len());
    let intp = &body[..dot_idx];
    let mut chars: Vec<char> = intp.chars().collect();
    chars.reverse();
    let mut out: Vec<char> = Vec::new();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(*c);
    }
    out.reverse();
    let mut result: String = out.into_iter().collect();
    if dot_idx < body.len() {
        result.push_str(&body[dot_idx..]);
    }
    if neg { result.insert(0, '-'); }
    result
}

fn apply_padding(s: &str, padding: isize) -> String {
    if padding == 0 { return s.to_string(); }
    let n = padding.unsigned_abs() as usize;
    if s.len() >= n { return s.to_string(); }
    let pad = " ".repeat(n - s.len());
    if padding > 0 {
        format!("{}{}", s, pad)
    } else {
        format!("{}{}", pad, s)
    }
}
```

### Step 12: numfmt 构建

```
cd /d d:\Work\rust\numfmt && cargo build --release --offline
```

---

## Part D: 冒烟测试（Step 13）

① csplit 按模式分割：
```
cd /d d:\Work\rust
Set-Content -Path test-csplit.txt -Value "line1`nline2`nXXX`nline3`nline4`nZZZ`nline5" -NoNewline
.\csplit\target\release\csplit.exe -k test-csplit.txt "/XXX/" "/ZZZ/"
dir xx??
Get-Content xx00; echo "---"; Get-Content xx01; echo "---"; Get-Content xx02
Remove-Item xx??,test-csplit.txt -ErrorAction SilentlyContinue
```
Expected: xx00 = line1+line2；xx01 = XXX 行到 ZZZ 行前；xx02 = ZZZ 行到最后；每个文件字节数分别被输出。

② pr 基本分页（带页眉+行号）：
```
cd /d d:\Work\rust
for ($i=1; $i -le 50; $i++) { echo "line $i" } | .\pr\target\release\pr.exe -n " " -h "demo" | Select-Object -First 15
```
Expected: 前 15 行含页眉（日期 + "demo" 居中等号分隔）+ 行号（带空格间隔）格式对齐。

③ numfmt iec-i（验证 PRE-FLIGHT i 后缀）+ si 对比：
```
cd /d d:\Work\rust\numfmt
.\target\release\numfmt.exe --to=iec-i 1536
.\target\release\numfmt.exe --to=iec 1536
.\target\release\numfmt.exe --to=si 1500
```
Expected:
- iec-i 1536: "1.5Ki" 或包含 "Ki"（1024 缩放 + 追加 i）
- iec 1536: "1.5K"（无 i）
- si 1500: "1.5K"（1000 缩放）

④ numfmt 反向 --from 解析：
```
.\target\release\numfmt.exe --from=iec-i 1.5Ki
```
Expected: ≈ 1536。

---

## Part E: 提交（Step 14）

```
cd /d d:\Work\rust
git add csplit/ pr/ numfmt/
git commit -m "feat: add csplit pr numfmt (iec-i preflight fixed) - round 7"
```

## Constraints

- 9 files only（3 × Cargo.toml/cli.rs/main.rs）。
- crate 名与目录一致（csplit/pr/numfmt）。
- cli.rs 全部 disable_version_flag=true，short='v' 自定义 version。
- main.rs 开头 `mod cli;` + `use clap::Parser;`。
- 错误前缀 eprintln!("csplit: ...")/("pr: ...")/("numfmt: ...")；std::process::exit。
- 不加多余注释；按 brief 逐字转录。
- numfmt 的 format_numeric() 中 iec-i 分支必须显式 append "i"，即必须看到 `if matches!(to, To::IecI) { "i" } else { "" }` 这样的代码块——这是 Pre-flight Fix。
- 不提交 target/、Cargo.lock。
