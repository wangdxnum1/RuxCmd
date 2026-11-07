//! Conservative Windows command-line sizing. No shell parsing or execution.

pub const MAX_COMMAND_UNITS: usize = 8000;

/// Size when every argument is quoted using Windows CRT backslash rules.
/// Quoting everything is an upper bound for Command's optional quoting.
fn argument_units(argument: &str) -> usize {
    let mut units = 2;
    let mut slashes = 0;
    for unit in argument.encode_utf16() {
        if unit == b'\\' as u16 {
            slashes += 1;
        } else {
            units += if unit == b'"' as u16 {
                slashes * 2 + 2
            } else {
                slashes + 1
            };
            slashes = 0;
        }
    }
    units + slashes * 2
}

/// Includes the terminating NUL, separators, program and all arguments.
pub fn command_units<S: AsRef<str>>(arguments: &[S]) -> usize {
    1 + arguments
        .iter()
        .map(|a| argument_units(a.as_ref()))
        .sum::<usize>()
        + arguments.len().saturating_sub(1)
}

#[derive(Debug, Clone)]
pub struct CommandBudget {
    base: usize,
    used: usize,
}

impl CommandBudget {
    pub fn new<S: AsRef<str>>(fixed: &[S]) -> Result<Self, &'static str> {
        let base = command_units(fixed);
        if fixed.is_empty() || base > MAX_COMMAND_UNITS {
            return Err("fixed command exceeds the 8000 UTF-16 unit limit");
        }
        Ok(Self { base, used: base })
    }

    pub fn can_fit_item(&self, item: &str) -> bool {
        self.base + 1 + argument_units(item) <= MAX_COMMAND_UNITS
    }

    pub fn try_add(&mut self, item: &str) -> bool {
        let next = self.used + 1 + argument_units(item);
        if next > MAX_COMMAND_UNITS {
            return false;
        }
        self.used = next;
        true
    }

    pub fn reset(&mut self) {
        self.used = self.base;
    }
}
