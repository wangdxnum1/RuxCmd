//! Byte-safe line reading over any `BufRead`.
//!
//! GNU-style semantics: a line ends at `\n`; a `\r` immediately before it is
//! part of the terminator (stripped), while a `\r` on an unterminated final
//! line stays part of the content. Lines are raw bytes — non-UTF-8 input
//! passes through untouched and callers decide how to match (lossy) and what
//! to print (original bytes).

use std::io::{self, BufRead};

/// Iterator over the raw byte lines of a reader, terminators stripped.
///
/// ```
/// use std::io::BufReader;
/// let input = b"a\r\nb\rc\n".as_slice();
/// let mut lines = ruxcmd_core::lines::ByteLines::new(BufReader::new(input));
/// assert_eq!(lines.next().unwrap().unwrap(), b"a".as_slice());
/// assert_eq!(lines.next().unwrap().unwrap(), b"b\rc".as_slice());
/// assert!(lines.next().is_none());
/// ```
pub struct ByteLines<R: BufRead> {
    reader: R,
    done: bool,
    separator: u8,
}

impl<R: BufRead> ByteLines<R> {
    pub fn new(reader: R) -> Self {
        Self::with_separator(reader, b'\n')
    }

    /// Read records separated by `separator`. NUL mode preserves all CR/LF
    /// bytes inside records; LF mode retains the historical CRLF handling.
    pub fn with_separator(reader: R, separator: u8) -> Self {
        ByteLines {
            reader,
            done: false,
            separator,
        }
    }

    /// Reads one line, stripping the `\n` (and a preceding `\r`); an
    /// unterminated final line is returned as-is. `Ok(None)` at EOF.
    pub fn next_line(&mut self) -> io::Result<Option<Vec<u8>>> {
        if self.done {
            return Ok(None);
        }
        let mut buf = Vec::new();
        let read = self.reader.read_until(self.separator, &mut buf)?;
        if read == 0 {
            self.done = true;
            return Ok(None);
        }
        if buf.last() == Some(&self.separator) {
            buf.pop();
            if self.separator == b'\n' && buf.last() == Some(&b'\r') {
                buf.pop();
            }
        }
        Ok(Some(buf))
    }
}

impl<R: BufRead> Iterator for ByteLines<R> {
    type Item = io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_line() {
            Ok(Some(line)) => Some(Ok(line)),
            Ok(None) => None,
            Err(e) => {
                self.done = true;
                Some(Err(e))
            }
        }
    }
}

/// True when the buffer ended with `\n` (used by callers that need to know
/// whether the last line was terminated, e.g. for `\\ No newline` output).
pub fn ended_with_newline(buf: &[u8]) -> bool {
    buf.last() == Some(&b'\n')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Read};

    fn collect(input: &[u8]) -> Vec<Vec<u8>> {
        ByteLines::new(BufReader::new(input))
            .map(|l| l.unwrap())
            .collect()
    }

    #[test]
    fn crlf_terminators_are_stripped() {
        assert_eq!(collect(b"a\r\nb\r\n"), vec![b"a".to_vec(), b"b".to_vec()]);
    }

    #[test]
    fn bare_cr_is_content_not_terminator() {
        assert_eq!(collect(b"a\rb\n"), vec![b"a\rb".to_vec()]);
    }

    #[test]
    fn unterminated_final_line_keeps_trailing_cr() {
        assert_eq!(collect(b"a\nb\r"), vec![b"a".to_vec(), b"b\r".to_vec()]);
    }

    #[test]
    fn empty_input_yields_nothing() {
        assert!(collect(b"").is_empty());
    }

    #[test]
    fn non_utf8_bytes_pass_through() {
        assert_eq!(
            collect(b"\xff\xfe\nok\n"),
            vec![b"\xff\xfe".to_vec(), b"ok".to_vec()]
        );
    }

    #[test]
    fn blank_lines_are_empty_not_skipped() {
        assert_eq!(collect(b"\n\n"), vec![Vec::new(), Vec::new()]);
    }

    #[test]
    fn nul_records_keep_newlines_carriage_returns_and_invalid_bytes() {
        let records: Vec<_> =
            ByteLines::with_separator(BufReader::new(b"a\r\nb\0\xff\r\0last".as_slice()), 0)
                .map(Result::unwrap)
                .collect();
        assert_eq!(
            records,
            vec![b"a\r\nb".to_vec(), b"\xff\r".to_vec(), b"last".to_vec()]
        );
    }

    #[test]
    fn nul_empty_records_and_empty_input_are_distinct() {
        let records: Vec<_> = ByteLines::with_separator(BufReader::new(b"\0\0".as_slice()), 0)
            .map(Result::unwrap)
            .collect();
        assert_eq!(records, vec![Vec::<u8>::new(), Vec::new()]);
        assert!(ByteLines::with_separator(BufReader::new(b"".as_slice()), 0)
            .next()
            .is_none());
    }

    #[test]
    fn a_failed_partial_record_is_not_reported_as_success() {
        struct FailingReader;
        impl Read for FailingReader {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("injected failure"))
            }
        }
        let input = std::io::Cursor::new(b"ok\0partial").chain(FailingReader);
        let mut records = ByteLines::with_separator(BufReader::new(input), 0);
        assert_eq!(records.next().unwrap().unwrap(), b"ok");
        assert!(records.next().unwrap().is_err());
        assert!(records.next().is_none());
    }
}
