mod cli;

fn main() {
    let args = cli::Cli::new();
    std::thread::sleep(args.duration());
}
