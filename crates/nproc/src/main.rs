use clap::Parser;
use windows_sys::Win32::System::SystemInformation::GetSystemInfo;

/// Print the number of processing units - a Windows port of Linux nproc.
#[derive(Parser, Debug)]
#[command(name = "nproc", version, about)]
pub struct Args {
    /// Count all processors, ignoring affinity and tasks
    #[arg(long = "all")]
    pub all: bool,

    /// Ignore up to N processors
    #[arg(long = "ignore", value_name = "N")]
    pub ignore: Option<u64>,
}

fn main() {
    let args = Args::parse();

    let count: u64 = if args.all {
        unsafe {
            let mut info = std::mem::zeroed();
            GetSystemInfo(&mut info);
            info.dwNumberOfProcessors as u64
        }
    } else {
        std::thread::available_parallelism()
            .map(|n| n.get() as u64)
            .unwrap_or(1)
    };

    let result = match args.ignore {
        Some(n) => count.saturating_sub(n).max(1),
        None => count,
    };
    println!("{result}");
}
