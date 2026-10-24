use clap::Parser;
use std::path::PathBuf;

const AFTER_HELP: &str = "\
Windows notes:
  Supported -c directives: %a %A %b %B %d %D %f %F %g %h %i %m %n %s %t %T %u %W %x %X %y %Y %z %Z %%
  Device numbers (%d %D %t %T), inode (%i), owner (%u %g) and link count (%h) are
  placeholders (0/1) on Windows; %b counts 512-byte units as ceil(size/512).
  %Z (change time) falls back to the modification time; %x %y %z are rendered in
  the local timezone. Unknown directives fail with exit code 1.
  -f reports the volume block size and total/free/available blocks only.";

#[derive(Parser, Debug)]
#[command(name = "stat", version, about, disable_version_flag = true, after_help = AFTER_HELP)]
pub struct Args {
    #[arg(short = 'c', long = "format")]
    pub format: Option<String>,

    #[arg(short = 'f', long = "file-system")]
    pub file_system: bool,

    #[arg(short = 't', long = "terse")]
    pub terse: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
