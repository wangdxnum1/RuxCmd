mod cli;

use clap::Parser;
use cli::Args;
use std::env;
use std::path::Path;

fn main() {
    let args = Args::parse();

    for cmd in &args.commands {
        let mut found = false;
        print!("{}: ", cmd);

        let path_env = env::var("PATH").unwrap_or_default();
        for path in path_env.split(';') {
            let full_path = Path::new(path).join(cmd);
            let exe_path = full_path.with_extension("exe");

            if exe_path.exists() {
                print!("{} ", exe_path.display());
                found = true;
            } else if full_path.exists() {
                print!("{} ", full_path.display());
                found = true;
            }
        }

        let rust_tools = "D:\\develop\\rust-tools";
        let exe_path = Path::new(rust_tools).join(cmd).with_extension("exe");
        if exe_path.exists() {
            print!("{} ", exe_path.display());
            found = true;
        }

        if !found {
            print!("not found");
        }
        println!();
    }
}
