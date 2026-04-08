/// A single weather radar scan point from a [`Response::Radar`] (RADR) packet.
#[derive(Debug, Clone, PartialEq)]
pub struct RadarPoint {
    /// Longitude of the scan point in degrees.
    pub longitude: f32,
    /// Latitude of the scan point in degrees.
    pub latitude: f32,
    /// Cloud base in meters MSL.
    pub cloud_base: f32,
    /// Cloud tops in meters MSL.
    pub cloud_tops: f32,
    /// Cloud coverage ratio (0.0–1.0), as seen from above.
    pub cloud_ratio: f32,
    /// Precipitation ratio (0.0–1.0), as seen from above.
    pub precip_ratio: f32,
}
