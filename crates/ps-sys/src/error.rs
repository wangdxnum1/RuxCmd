#![cfg(windows)]

use windows::core::Error as Win32Error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Win32 error: {0}")]
    Win32(#[from] Win32Error),
    #[error("invalid PID {0}")]
    InvalidPid(u32),
    #[error("permission denied for PID {0}")]
    PermissionDenied(u32),
    #[error("process not found: PID {0}")]
    NotFound(u32),
    #[error("PEB read failed for PID {0}")]
    PebRead(u32),
    #[error("internal: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_displays_kind() {
        let e = Error::InvalidPid(42);
        assert_eq!(format!("{e}"), "invalid PID 42");
    }
}
