mod cli;

use clap::Parser;
use cli::Cli;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::{self, copy};
use std::path::Path;

fn main() -> io::Result<()> {
    let args = Cli::parse();

    let gz_path = Path::new(&args.file);

    if !gz_path.exists() {
        eprintln!("错误: 文件不存在 - {}", args.file);
        std::process::exit(1);
    }

    if !gz_path.extension().map_or(false, |ext| ext == "gz") {
        eprintln!("错误: 文件不是 .gz 格式 - {}", args.file);
        std::process::exit(1);
    }

    let output_path = match gz_path.file_stem() {
        Some(stem) => gz_path.with_file_name(stem),
        None => {
            eprintln!("错误: 无法确定输出文件名 - {}", args.file);
            std::process::exit(1);
        }
    };

    if output_path.exists() && !args.force {
        eprintln!("错误: 目标文件已存在 - {}", output_path.display());
        eprintln!("使用 -f 或 --force 强制覆盖");
        std::process::exit(1);
    }

    if args.verbose {
        println!("解压: {} -> {}", gz_path.display(), output_path.display());
    }

    let input_file = File::open(gz_path)?;
    let mut decoder = GzDecoder::new(input_file);
    let mut output_file = File::create(&output_path)?;

    copy(&mut decoder, &mut output_file)?;

    if args.verbose {
        println!("解压完成");
    }

    if !args.keep {
        fs::remove_file(gz_path)?;
        if args.verbose {
            println!("已删除原始文件: {}", gz_path.display());
        }
    }

    Ok(())
}
