use crate::codec::reader::Reader;
use crate::error::{Error, Result};
use crate::types::{data::DataOutput, dataref::DatarefValue, position::AircraftPosition, radar::RadarPoint};

/// A message received from X-Plane.
///
/// Obtain values by calling [`XPlaneClient::recv`].
#[derive(Debug, Clone)]
pub enum Response {
    /// Aircraft position data, received after calling [`XPlaneClient::request_position`].
    ///
    /// Field order in wire format: lon, lat, ele (doubles), then pitch/heading/roll/speeds/rates (floats).
    Position(AircraftPosition),

    /// Weather radar scan points, received after calling [`XPlaneClient::request_radar`].
    Radar(Vec<RadarPoint>),

    /// Dataref values, received after calling [`XPlaneClient::subscribe_dataref`].
    ///
    /// A single packet may contain values for multiple subscribed datarefs.
    DatarefValues(Vec<DatarefValue>),

    /// A data output item, received when data streaming is enabled via
    /// [`XPlaneClient::select_data`] or the X-Plane data output screen.
    Data(DataOutput),
}

impl Response {
    /// Decode a raw UDP packet from X-Plane into a `Response`.
    pub(crate) fn decode(buf: &[u8]) -> Result<Self> {
        if buf.len() < 5 {
            return Err(Error::InvalidResponse(format!(
                "packet too short: {} bytes",
                buf.len()
            )));
        }

        let tag: [u8; 4] = buf[..4].try_into().unwrap();
        // All packets start with a 5-byte header (4-char tag + 1 byte).
        let mut r = Reader::new(&buf[5..]);

        match &tag {
            b"RPOS" => {
                // wire order: lon, lat, ele (f64), then y_agl, pitch, heading, roll, speed_east, speed_up, speed_south, roll_rate, pitch_rate, yaw_rate (f32).
                Ok(Response::Position(AircraftPosition {
                    longitude: r.read_f64()?,
                    latitude: r.read_f64()?,
                    elevation: r.read_f64()?,
                    above_ground: r.read_f32()?,
                    pitch: r.read_f32()?,
                    heading: r.read_f32()?,
                    roll: r.read_f32()?,
                    speed_east: r.read_f32()?,
                    speed_up: r.read_f32()?,
                    speed_south: r.read_f32()?,
                    roll_rate: r.read_f32()?,
                    pitch_rate: r.read_f32()?,
                    yaw_rate: r.read_f32()?,
                }))
            }

            b"RADR" => {
                // wire order per point: lon, lat, cloud_base, cloud_tops, cloud_ratio, precip_ratio (f32).
                let mut points = Vec::new();
                while r.remaining() >= 24 {
                    points.push(RadarPoint {
                        longitude: r.read_f32()?,
                        latitude: r.read_f32()?,
                        cloud_base: r.read_f32()?,
                        cloud_tops: r.read_f32()?,
                        cloud_ratio: r.read_f32()?,
                        precip_ratio: r.read_f32()?,
                    });
                }
                Ok(Response::Radar(points))
            }

            b"RREF" => {
                // wire: repeated (i32 index, f32 value) pairs.
                let mut values = Vec::new();
                while r.remaining() >= 8 {
                    values.push(DatarefValue {
                        index: r.read_i32()?,
                        value: r.read_f32()?,
                    });
                }
                Ok(Response::DatarefValues(values))
            }

            b"DATA" => {
                // wire: i32 index, then 8 × f32.
                let index = r.read_i32()?;
                let mut values = [0f32; 8];
                for v in &mut values {
                    *v = r.read_f32()?;
                }
                Ok(Response::Data(DataOutput { index, values }))
            }

            _ => Err(Error::UnknownTag(tag)),
        }
    }
}
