/// One data output item from a [`Response::Data`] (DATA) packet.
#[derive(Debug, Clone, PartialEq)]
pub struct DataOutput {
    /// Data output screen index.
    pub index: i32,
    /// The 8 values for this item (`-999.0` means "not set / use default").
    pub values: [f32; 8],
}
