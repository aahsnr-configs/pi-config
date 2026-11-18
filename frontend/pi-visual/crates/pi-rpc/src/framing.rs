//! Bounded, LF-only framing for a blocking Pi stdout pipe.
//!
//! Bad records are drained through LF before returning an error. I/O errors are
//! terminal for the stream; protocol errors allow reading the following record.

use crate::{Incoming, MAX_RPC_LINE_BYTES, ProtocolError, parse_line};
use std::{
    error::Error,
    fmt,
    io::{self, BufRead},
};

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Protocol(ProtocolError),
    InvalidUtf8,
    TruncatedRecord,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => f.write_str("RPC stream I/O failure"),
            Self::Protocol(error) => error.fmt(f),
            Self::InvalidUtf8 => f.write_str("RPC record is not UTF-8"),
            Self::TruncatedRecord => f.write_str("RPC stream ended before record LF"),
        }
    }
}

impl Error for ReadError {}

pub struct JsonlReader<R> {
    reader: R,
}

impl<R: BufRead> JsonlReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Return one complete parsed record, or `None` at a clean EOF.
    /// The byte limit excludes LF and one optional trailing CR. Retained record
    /// bytes never exceed that limit; Vec capacity may reserve additional memory.
    pub fn next_record(&mut self) -> Result<Option<Incoming>, ReadError> {
        let mut record = Vec::new();
        let mut oversized = false;
        loop {
            let bytes = match self.reader.fill_buf() {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                other => other.map_err(ReadError::Io)?,
            };
            if bytes.is_empty() {
                return if oversized {
                    Err(ReadError::Protocol(ProtocolError::LineTooLong))
                } else if record.is_empty() {
                    Ok(None)
                } else {
                    Err(ReadError::TruncatedRecord)
                };
            }
            let boundary = bytes.iter().position(|byte| *byte == b'\n');
            let count = boundary.unwrap_or(bytes.len());
            if !oversized {
                if count > (MAX_RPC_LINE_BYTES + 1).saturating_sub(record.len()) {
                    oversized = true;
                    record.clear();
                } else {
                    record.extend_from_slice(&bytes[..count]);
                }
            }
            self.reader.consume(count + usize::from(boundary.is_some()));
            if boundary.is_some() {
                if record.last() == Some(&b'\r') {
                    record.pop();
                }
                if oversized || record.len() > MAX_RPC_LINE_BYTES {
                    return Err(ReadError::Protocol(ProtocolError::LineTooLong));
                }
                let line = std::str::from_utf8(&record).map_err(|_| ReadError::InvalidUtf8)?;
                return parse_line(line).map(Some).map_err(ReadError::Protocol);
            }
        }
    }
}
