mod cli;

use clap::Parser;
use cli::Cli;
use flate2::read::MultiGzDecoder;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

fn main() -> io::Result<()> {
    let args = Cli::parse();

    let gz_path = Path::new(&args.file);

    if !gz_path.exists() {
        eprintln!("gunzip: {}: No such file or directory", args.file);
        std::process::exit(1);
    }

    if !gz_path.extension().map_or(false, |ext| ext == "gz") {
        eprintln!("gunzip: {}: not in gzip format", args.file);
        std::process::exit(1);
    }

    let input_file = File::open(gz_path)?;

    if args.list {
        return list_stats(input_file, gz_path);
    }

    if args.stdout {
        // -c writes to stdout and never removes the source (GNU implies
        // keep); multi-member streams decompress fully.
        let mut decoder = MultiGzDecoder::new(input_file);
        let stdout = io::stdout();
        let mut out = stdout.lock();
        io::copy(&mut decoder, &mut out)?;
        out.flush()?;
        return Ok(());
    }

    let output_path = match gz_path.file_stem() {
        Some(stem) => gz_path.with_file_name(stem),
        None => {
            eprintln!("gunzip: {}: cannot determine output file name", args.file);
            std::process::exit(1);
        }
    };

    if output_path.exists() && !args.force {
        eprintln!("gunzip: {} already exists", output_path.display());
        eprintln!("use -f or --force to overwrite");
        std::process::exit(1);
    }

    if args.verbose {
        println!(
            "decompressing: {} -> {}",
            gz_path.display(),
            output_path.display()
        );
    }

    let mut decoder = MultiGzDecoder::new(input_file);
    let mut output_file = File::create(&output_path)?;

    io::copy(&mut decoder, &mut output_file)?;

    if args.verbose {
        println!("decompression complete");
    }

    if !args.keep {
        fs::remove_file(gz_path)?;
        if args.verbose {
            println!("removed original file: {}", gz_path.display());
        }
    }

    Ok(())
}

/// `-l`: prints compressed size (file length), uncompressed size (total
/// decompressed bytes across all members) and the ratio, GNU style.
fn list_stats(mut input: File, gz_path: &Path) -> io::Result<()> {
    let compressed = input.metadata()?.len();

    let mut decoder = MultiGzDecoder::new(&mut input);
    let mut uncompressed: u64 = 0;
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = decoder.read(&mut buf)?;
        if n == 0 {
            break;
        }
        uncompressed += n as u64;
    }

    let ratio = if uncompressed > 0 {
        ((compressed as f64 - uncompressed as f64) / uncompressed as f64) * 100.0
    } else {
        0.0
    };

    let out_name = gz_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| gz_path.display().to_string());

    println!("  compressed uncompressed  ratio uncompressed_name");
    println!(
        "{:>12} {:>12} {:>6} {}",
        compressed,
        uncompressed,
        format!("{:.1}%", ratio),
        out_name
    );

    Ok(())
}
