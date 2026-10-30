use clap::Parser;

/// Print a numeric host identifier - a Windows port of Linux hostid.
/// The identifier is a stable 32-bit hash of the machine's DNS host name,
/// printed as eight lowercase hex digits.
#[derive(Parser, Debug)]
#[command(name = "hostid", version, about)]
pub struct Args {}

fn main() {
    let _ = Args::parse();

    let host = std::env::var("COMPUTERNAME").unwrap_or_default();
    let hash: u32 = fnv1a_32(host.as_bytes());
    println!("{:08x}", hash);
}

fn fnv1a_32(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 0x811C9DC5;
    for &b in bytes {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}
