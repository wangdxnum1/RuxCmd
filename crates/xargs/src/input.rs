use ruxcmd_core::command_batch::MAX_COMMAND_UNITS;
use std::io::BufRead;

pub struct Items<R> {
    reader: R,
    null: bool,
    lines: bool,
}

impl<R: BufRead> Items<R> {
    pub fn new(reader: R, null: bool, lines: bool) -> Self {
        Self {
            reader,
            null,
            lines,
        }
    }

    pub fn next(&mut self) -> Result<Option<String>, String> {
        let mut bytes = Vec::new();
        let mut active = false;
        let mut quote = None;
        let mut escaped = false;
        loop {
            let byte = self
                .reader
                .fill_buf()
                .map_err(|e| format!("error reading stdin: {e}"))?
                .first()
                .copied();
            let Some(byte) = byte else {
                if quote.is_some() {
                    return Err("unmatched quote".into());
                }
                if escaped {
                    return Err("incomplete escape at end of input".into());
                }
                return if active {
                    decode(bytes).map(Some)
                } else {
                    Ok(None)
                };
            };
            self.reader.consume(1);
            if self.null {
                if byte == 0 {
                    return decode(bytes).map(Some);
                }
                active = true;
                bytes.push(byte);
            } else {
                if byte == 0 {
                    return Err("NUL in ordinary input; use -0".into());
                }
                if escaped {
                    bytes.push(byte);
                    active = true;
                    escaped = false;
                } else if let Some(delimiter) = quote {
                    if byte == delimiter {
                        quote = None;
                    } else if byte == b'\n' {
                        return Err("unmatched quote".into());
                    } else {
                        bytes.push(byte);
                    }
                } else {
                    match byte {
                        b'\'' | b'"' => {
                            active = true;
                            quote = Some(byte);
                        }
                        b'\\' => {
                            active = true;
                            escaped = true;
                        }
                        b'\n' => {
                            if active {
                                return decode(bytes).map(Some);
                            }
                        }
                        b' ' | b'\t' if !self.lines => {
                            if active {
                                return decode(bytes).map(Some);
                            }
                        }
                        b' ' | b'\t' if !active => {}
                        _ => {
                            active = true;
                            bytes.push(byte);
                        }
                    }
                }
            }
            // UTF-8 uses at most four bytes per scalar. Bound even a single
            // unterminated item before decoding and exact UTF-16 sizing.
            if bytes.len() > MAX_COMMAND_UNITS * 4 {
                return Err("input item exceeds command-line limit".into());
            }
        }
    }
}

fn decode(bytes: Vec<u8>) -> Result<String, String> {
    String::from_utf8(bytes).map_err(|_| "invalid UTF-8 input".into())
}
