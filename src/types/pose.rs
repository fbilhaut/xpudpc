/// World position and orientation for an aircraft or object.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub lat: f64,
    pub lon: f64,
    pub ele: f64,
    pub heading: f32,
    pub pitch: f32,
    pub roll: f32,
}
