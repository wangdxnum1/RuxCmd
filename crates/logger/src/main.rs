use clap::Parser;
use std::io::Read;

/// Default event source registered in the Application log.
const DEFAULT_SOURCE: &str = "RuxCmd";
/// Event ID used for every entry. Without a registered message DLL Windows
/// cannot render a description for this ID.
const EVENT_ID: u32 = 1;
/// Test/diagnostic hook that forces the stderr fallback path.
const FORCE_FALLBACK_ENV: &str = "RUXCMD_LOGGER_FORCE_STDERR";

/// Write messages to the Windows event log — a Windows port of Linux logger.
#[derive(Parser, Debug)]
#[command(name = "logger", version, about)]
pub struct Args {
    /// Print the message to stderr as well
    #[arg(short = 's')]
    pub stderr: bool,

    /// Tag the message and use it as the event source name
    #[arg(short = 't', long = "tag", value_name = "TAG")]
    pub tag: Option<String>,

    /// Syslog priority: FACILITY.PRIORITY or PRIORITY (default user.notice)
    #[arg(
        short = 'p',
        long = "priority",
        value_name = "PRIORITY",
        default_value = "user.notice",
        value_parser = parse_priority
    )]
    pub priority: EventType,

    /// Messages to log (default: read from stdin)
    #[arg(value_name = "MESSAGE")]
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Error,
    Warning,
    Information,
}

/// Map a syslog priority name to a Windows event type. An optional
/// `FACILITY.` prefix is accepted and ignored, like GNU logger.
fn event_type_for_priority(priority: &str) -> Option<EventType> {
    let name = priority.rsplit('.').next().unwrap_or(priority);
    match name.to_ascii_lowercase().as_str() {
        "emerg" | "alert" | "crit" | "err" | "error" | "panic" => Some(EventType::Error),
        "warning" | "warn" => Some(EventType::Warning),
        "notice" | "info" | "debug" => Some(EventType::Information),
        _ => None,
    }
}

fn parse_priority(value: &str) -> Result<EventType, String> {
    event_type_for_priority(value).ok_or_else(|| {
        format!(
            "invalid priority '{}': use FACILITY.PRIORITY with PRIORITY = emerg, alert, crit, err, warning, notice, info, debug",
            value
        )
    })
}

fn main() {
    let args = Args::parse();
    let explicit_tag = args.tag.clone();
    let message_tag = explicit_tag
        .clone()
        .unwrap_or_else(|| std::env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string()));
    let source = explicit_tag.unwrap_or_else(|| DEFAULT_SOURCE.to_string());

    let text = if args.messages.is_empty() {
        let mut buf = String::new();
        let _ = std::io::stdin().read_to_string(&mut buf);
        buf
    } else {
        args.messages.join(" ")
    };

    let mut sink = EventSink::new(message_tag, source, args.priority, args.stderr);
    for line in text.lines() {
        sink.write(line);
    }
}

/// Writes one entry per line to the Application log. Any failure (no
/// Windows support, RegisterEventSourceW/ReportEventW error, forced test
/// hook) switches permanently to stderr so no message is ever dropped.
struct EventSink {
    message_tag: String,
    source: String,
    priority: EventType,
    echo_stderr: bool,
    state: SinkState,
}

enum SinkState {
    Unopened,
    Ready(eventlog::EventSource),
    Failed,
}

impl EventSink {
    fn new(message_tag: String, source: String, priority: EventType, echo_stderr: bool) -> Self {
        Self {
            message_tag,
            source,
            priority,
            echo_stderr,
            state: SinkState::Unopened,
        }
    }

    fn write(&mut self, line: &str) {
        let entry = format!("{}: {}", self.message_tag, line);
        if self.echo_stderr {
            eprintln!("{}", entry);
        }
        if matches!(self.state, SinkState::Failed) {
            if !self.echo_stderr {
                eprintln!("{}", entry);
            }
            return;
        }

        // `replace` releases the borrow on `state` so a fresh handle can be
        // installed while the old one is dropped (and deregistered).
        let outcome = match std::mem::replace(&mut self.state, SinkState::Failed) {
            SinkState::Ready(source) => {
                let result = source.report(self.priority, &entry);
                self.state = SinkState::Ready(source);
                result
            }
            SinkState::Unopened => {
                if std::env::var_os(FORCE_FALLBACK_ENV).is_some() {
                    Err(format!("event log disabled by {}", FORCE_FALLBACK_ENV))
                } else {
                    match eventlog::EventSource::open(&self.source) {
                        Ok(source) => {
                            let result = source.report(self.priority, &entry);
                            self.state = SinkState::Ready(source);
                            result
                        }
                        Err(e) => Err(e),
                    }
                }
            }
            SinkState::Failed => Ok(()),
        };

        if let Err(reason) = outcome {
            eprintln!("logger: {}; writing to stderr instead", reason);
            self.state = SinkState::Failed;
            if !self.echo_stderr {
                eprintln!("{}", entry);
            }
        }
    }
}

#[cfg(windows)]
mod eventlog {
    use super::{EVENT_ID, EventType};
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Security::PSID;
    use windows::Win32::System::EventLog::{
        DeregisterEventSource, EVENTLOG_ERROR_TYPE, EVENTLOG_INFORMATION_TYPE,
        EVENTLOG_WARNING_TYPE, RegisterEventSourceW, ReportEventW,
    };
    use windows::core::PCWSTR;

    pub struct EventSource {
        handle: HANDLE,
    }

    impl EventSource {
        pub fn open(source: &str) -> Result<Self, String> {
            let name = wide(source);
            unsafe {
                RegisterEventSourceW(PCWSTR::null(), PCWSTR(name.as_ptr()))
                    .map(|handle| Self { handle })
                    .map_err(|e| format!("cannot open event source '{}': {}", source, e))
            }
        }

        pub fn report(&self, kind: EventType, message: &str) -> Result<(), String> {
            let text = wide(message);
            let strings = [PCWSTR(text.as_ptr())];
            let event_type = match kind {
                EventType::Error => EVENTLOG_ERROR_TYPE,
                EventType::Warning => EVENTLOG_WARNING_TYPE,
                EventType::Information => EVENTLOG_INFORMATION_TYPE,
            };
            unsafe {
                ReportEventW(
                    self.handle,
                    event_type,
                    0,
                    EVENT_ID,
                    PSID(std::ptr::null_mut()),
                    0,
                    Some(&strings),
                    None,
                )
                .map_err(|e| format!("ReportEventW failed: {}", e))
            }
        }
    }

    impl Drop for EventSource {
        fn drop(&mut self) {
            unsafe {
                let _ = DeregisterEventSource(self.handle);
            }
        }
    }

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(not(windows))]
mod eventlog {
    use super::EventType;

    pub struct EventSource;

    impl EventSource {
        pub fn open(_source: &str) -> Result<Self, String> {
            Err("Windows event log is unavailable on this platform".to_string())
        }

        pub fn report(&self, _kind: EventType, _message: &str) -> Result<(), String> {
            Err("Windows event log is unavailable on this platform".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EventType, event_type_for_priority};

    #[test]
    fn error_priorities_map_to_error_type() {
        for name in [
            "emerg", "alert", "crit", "err", "error", "panic", "user.err",
        ] {
            assert_eq!(
                event_type_for_priority(name),
                Some(EventType::Error),
                "{name}"
            );
        }
    }

    #[test]
    fn warning_priorities_map_to_warning_type() {
        for name in ["warning", "warn", "user.warning"] {
            assert_eq!(
                event_type_for_priority(name),
                Some(EventType::Warning),
                "{name}"
            );
        }
    }

    #[test]
    fn remaining_priorities_map_to_information_type() {
        for name in ["notice", "info", "debug", "user.notice", "user.info"] {
            assert_eq!(
                event_type_for_priority(name),
                Some(EventType::Information),
                "{name}"
            );
        }
    }

    #[test]
    fn names_are_case_insensitive_and_unknown_ones_are_rejected() {
        assert_eq!(
            event_type_for_priority("USER.WARNING"),
            Some(EventType::Warning)
        );
        assert_eq!(event_type_for_priority("bogus"), None);
        assert_eq!(event_type_for_priority("user."), None);
    }
}
