pub struct Args {
    pub null_delimited: bool,
    pub max_args: usize,
    pub interactive: bool,
    pub verbose: bool,
    pub replace: Option<String>,
    pub no_run_if_empty: bool,
    pub command: Vec<String>,
}

const HELP: &str = "Build and execute command lines from standard input

Usage: xargs [OPTIONS] [COMMAND [ARG]...]

Options:
  -0, --null                  Read NUL-delimited items literally
  -n, --max-args NUMBER        Use at most NUMBER items per command
  -I, --replace[=STR]          Replace all occurrences of STR (default {})
  -i                          Equivalent to --replace={}
  -r, --no-run-if-empty        Do not run with empty input
  -p, --interactive            Prompt before each command
  -t, --verbose                Print each command to stderr
  -V, --version                Print version
  -h, --help                   Print help

Input is UTF-8. Ordinary input supports xargs quotes/backslashes, not shell
syntax. Use find -print0 with -0 for Windows paths. Commands run directly;
shell builtins require an explicit shell. Batches use at most 8000 UTF-16
units, including quoting and fixed arguments. Child stdin is closed.";

impl Args {
    pub fn parse() -> Result<Self, String> {
        let argv: Vec<String> = std::env::args().skip(1).collect();
        let mut args = Self {
            null_delimited: false,
            max_args: usize::MAX,
            interactive: false,
            verbose: false,
            replace: None,
            no_run_if_empty: false,
            command: Vec::new(),
        };
        let mut i = 0;
        while i < argv.len() {
            let word = &argv[i];
            if word == "--" {
                i += 1;
                break;
            }
            if !word.starts_with('-') || word == "-" {
                break;
            }
            if let Some(long) = word.strip_prefix("--") {
                let (name, value) = long
                    .split_once('=')
                    .map_or((long, None), |(k, v)| (k, Some(v)));
                match name {
                    "help" if value.is_none() => {
                        println!("{HELP}");
                        std::process::exit(0);
                    }
                    "version" if value.is_none() => {
                        println!("xargs 0.1.0");
                        std::process::exit(0);
                    }
                    "null" if value.is_none() => args.null_delimited = true,
                    "no-run-if-empty" if value.is_none() => args.no_run_if_empty = true,
                    "verbose" if value.is_none() => args.verbose = true,
                    "interactive" if value.is_none() => args.interactive = true,
                    "replace" => args.set_replace(value.unwrap_or("{}"))?,
                    "max-args" => {
                        let value = match value {
                            Some(v) => v,
                            None => next_value(&argv, &mut i, word)?,
                        };
                        args.set_max(value)?;
                    }
                    _ => return Err(format!("unrecognized option '{word}'")),
                }
            } else {
                for (offset, flag) in word[1..].char_indices() {
                    match flag {
                        '0' => args.null_delimited = true,
                        'r' => args.no_run_if_empty = true,
                        'p' => args.interactive = true,
                        't' => args.verbose = true,
                        'h' => {
                            println!("{HELP}");
                            std::process::exit(0);
                        }
                        'V' => {
                            println!("xargs 0.1.0");
                            std::process::exit(0);
                        }
                        'I' | 'n' | 'i' => {
                            let tail = &word[offset + 2..];
                            let value = if !tail.is_empty() {
                                tail
                            } else if flag == 'i' {
                                "{}"
                            } else {
                                next_value(&argv, &mut i, word)?
                            };
                            if flag == 'n' {
                                args.set_max(value)?;
                            } else {
                                args.set_replace(value)?;
                            }
                            break;
                        }
                        _ => return Err(format!("invalid option -- '{flag}'")),
                    }
                }
            }
            i += 1;
        }
        args.command = argv[i..].to_vec();
        Ok(args)
    }

    fn set_max(&mut self, value: &str) -> Result<(), String> {
        let count = value
            .parse::<usize>()
            .ok()
            .filter(|n| *n > 0)
            .ok_or_else(|| format!("invalid number of arguments: '{value}'"))?;
        if self.replace.is_some() && count == 1 {
            return Ok(());
        }
        if self.replace.take().is_some() {
            eprintln!(
                "xargs: warning: options --replace and --max-args/-n are mutually exclusive, ignoring previous --replace value"
            );
        }
        self.max_args = count;
        Ok(())
    }

    fn set_replace(&mut self, value: &str) -> Result<(), String> {
        if value.is_empty() {
            return Err("replacement string must not be empty".into());
        }
        if self.max_args != usize::MAX {
            eprintln!(
                "xargs: warning: options --max-args and --replace/-I/-i are mutually exclusive, ignoring previous --max-args value"
            );
        }
        self.max_args = usize::MAX;
        self.replace = Some(value.to_string());
        Ok(())
    }
}

fn next_value<'a>(argv: &'a [String], i: &mut usize, option: &str) -> Result<&'a str, String> {
    *i += 1;
    argv.get(*i)
        .map(String::as_str)
        .ok_or_else(|| format!("option '{option}' requires an argument"))
}
