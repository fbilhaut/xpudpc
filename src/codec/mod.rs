//! Low-level binary encoding and decoding helpers for the X-Plane UDP protocol.
//!
//! All values are little-endian (X-Plane runs on x86/ARM, both LE).

pub mod writer;
pub mod reader;
