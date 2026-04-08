use crate::error::{Error, Result};

/// A cursor for reading little-endian binary data from a byte slice.
pub(crate) struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(buf: &'a [u8]) -> Self {
        Reader { buf, pos: 0 }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    fn read_bytes<const N: usize>(&mut self) -> Result<[u8; N]> {
        if self.pos + N > self.buf.len() {
            return Err(Error::InvalidResponse("unexpected end of data".into()));
        }
        let arr: [u8; N] = self.buf[self.pos..self.pos + N].try_into().unwrap(); // bounds checked above.
        self.pos += N;
        Ok(arr)
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8> {
        Ok(u8::from_le_bytes(self.read_bytes::<1>()?))
    }

    pub(crate) fn read_u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.read_bytes::<2>()?))
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.read_bytes::<4>()?))
    }

    pub(crate) fn read_i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.read_bytes::<4>()?))
    }

    pub(crate) fn read_f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.read_bytes::<4>()?))
    }

    pub(crate) fn read_f64(&mut self) -> Result<f64> {
        Ok(f64::from_le_bytes(self.read_bytes::<8>()?))
    }

    /// Read exactly `len` bytes as a null-terminated (or full) UTF-8 string.
    pub(crate) fn read_str(&mut self, len: usize) -> Result<String> {
        if self.pos + len > self.buf.len() {
            return Err(Error::InvalidResponse("unexpected end of data".into()));
        }
        let raw = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        String::from_utf8(raw[..end].to_vec())
            .map_err(|e| Error::InvalidResponse(format!("invalid UTF-8 in string: {e}")))
    }
}