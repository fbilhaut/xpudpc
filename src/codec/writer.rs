use crate::error::{Error, Result};

/// Write the 5-byte packet header: 4-char tag + null byte.
pub(crate) fn write_header(buf: &mut Vec<u8>, tag: &[u8; 4]) {
    buf.extend_from_slice(tag);
    buf.push(0);
}

pub(crate) fn write_i32(buf: &mut Vec<u8>, v: i32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

pub(crate) fn write_f32(buf: &mut Vec<u8>, v: f32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

pub(crate) fn write_f64(buf: &mut Vec<u8>, v: f64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// Write a string into exactly `total_len` bytes, null-terminated and zero-padded.
pub(crate) fn write_str_zero(
    buf: &mut Vec<u8>,
    s: &str,
    total_len: usize,
    field: &'static str,
) -> Result<()> {
    let bytes = s.as_bytes();
    if bytes.len() >= total_len {
        return Err(Error::StringTooLong {
            field,
            max: total_len - 1,
            actual: bytes.len(),
        });
    }
    buf.extend_from_slice(bytes);
    for _ in 0..(total_len - bytes.len()) {
        buf.push(0);
    }
    Ok(())
}

/// Write a string into exactly `total_len` bytes, null-terminated then space-padded.
///
/// Used by DREF, which requires: `<string>\0<spaces…>` to fill the field.
pub(crate) fn write_str_spaces(
    buf: &mut Vec<u8>,
    s: &str,
    total_len: usize,
    field: &'static str,
) -> Result<()> {
    let bytes = s.as_bytes();
    if bytes.len() >= total_len {
        return Err(Error::StringTooLong {
            field,
            max: total_len - 1,
            actual: bytes.len(),
        });
    }
    buf.extend_from_slice(bytes);
    buf.push(0); // null terminator
    for _ in 0..(total_len - bytes.len() - 1) {
        buf.push(b' ');
    }
    Ok(())
}
