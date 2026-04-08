/// A single dataref value from a [`Response::DatarefValues`] (RREF) packet.
#[derive(Debug, Clone, PartialEq)]
pub struct DatarefValue {
    /// The index you assigned when calling [`XPlaneClient::subscribe_dataref`].
    pub index: i32,
    /// Current value of the dataref (floats and integers are both sent as f32).
    pub value: f32,
}
