/// Aircraft position received from an [`Response::Position`] (RPOS) packet.
#[derive(Debug, Clone, PartialEq)]
pub struct AircraftPosition {
    /// Longitude in degrees.
    pub longitude: f64,
    /// Latitude in degrees.
    pub latitude: f64,
    /// Elevation above sea level in meters.
    pub elevation: f64,
    /// Elevation above terrain in meters.
    pub above_ground: f32,
    /// Pitch in degrees.
    pub pitch: f32,
    /// True heading in degrees.
    pub heading: f32,
    /// Roll in degrees.
    pub roll: f32,
    /// Speed eastward in m/s.
    pub speed_east: f32,
    /// Speed upward in m/s.
    pub speed_up: f32,
    /// Speed southward in m/s.
    pub speed_south: f32,
    /// Roll rate in radians per second.
    pub roll_rate: f32,
    /// Pitch rate in radians per second.
    pub pitch_rate: f32,
    /// Yaw rate in radians per second.
    pub yaw_rate: f32,
}
