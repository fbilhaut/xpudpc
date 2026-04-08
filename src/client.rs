use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{ToSocketAddrs, UdpSocket, lookup_host};
use crate::codec::writer::*;
use crate::error::{Error, Result};
use crate::response::Response;
use crate::types::{placement::PlacementConfig, situation::SituationOp};

// Receive buffer size
const RECV_BUF: usize = 4096;

// String field sizes as defined in the X-Plane protocol.
const STR_DIM: usize = 500; // general path / string fields
const NET_STR_DIM: usize = 150; // network-length path fields (ACFN, SIMO)
const DATAREF_DIM: usize = 400; // RREF dataref string field

/// A client for communicating with X-Plane over UDP.
///
/// Create with [`XPlaneClient::connect`], then call the send methods and
/// [`XPlaneClient::recv`] to exchange data with a running X-Plane instance.
///
/// X-Plane listens on **port 49000** by default.
///
/// # Example
///
/// ```no_run
/// use xpudpc::{XPlaneClient, Response};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> xpudpc::Result<()> {
///     let client = XPlaneClient::connect("127.0.0.1:49000").await?;
///
///     client.subscribe_dataref(10, 0, "sim/cockpit2/gauges/indicators/airspeed_kts_pilot").await?;
///     client.request_position(30.0).await?;
///
///     loop {
///         match client.recv().await? {
///             Response::DatarefValues(refs) => {
///                 for r in refs {
///                     println!("dataref {}: {:.1}", r.index, r.value);
///                 }
///             }
///             Response::Position(pos) => {
///                 println!("lat={:.4} lon={:.4} alt={:.0}m", pos.latitude, pos.longitude, pos.elevation);
///             }
///             _ => {}
///         }
///     }
/// }
/// ```
pub struct XPlaneClient {
    socket: UdpSocket,
    xplane_addr: SocketAddr,
}

impl XPlaneClient {
    /// Connect to X-Plane at the given address.
    ///
    /// The client binds to an ephemeral local port; X-Plane sends replies back to that same port.
    pub async fn connect(xplane_addr: impl ToSocketAddrs) -> Result<Self> {
        let xplane_addr = lookup_host(xplane_addr)
            .await?
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no address resolved"))?;
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        Ok(XPlaneClient {
            socket,
            xplane_addr,
        })
    }

    /// Receive the next packet from X-Plane.
    pub async fn recv(&self) -> Result<Response> {
        let mut buf = [0u8; RECV_BUF];
        let (n, _src) = self.socket.recv_from(&mut buf).await?;
        Response::decode(&buf[..n])
    }

    /// Receive the next packet, returning an error if `duration` elapses first.
    pub async fn recv_timeout(&self, duration: Duration) -> Result<Response> {
        tokio::time::timeout(duration, self.recv())
            .await
            .map_err(|_| Error::Io(io::Error::new(io::ErrorKind::TimedOut, "recv timed out")))?
    }

    /// Send a command whose only payload is a frequency as a null-terminated ASCII string.
    /// X-Plane uses this format for RPOS and RADR to avoid byte-order issues.
    async fn send_freq_cmd(&self, tag: &[u8; 4], freq: u32) -> Result<()> {
        let s = freq.to_string();
        let mut buf = Vec::with_capacity(5 + s.len() + 1);
        write_header(&mut buf, tag);
        buf.extend_from_slice(s.as_bytes());
        buf.push(0);
        self.send(buf).await
    }

    async fn send(&self, buf: Vec<u8>) -> Result<()> {
        self.socket.send_to(&buf, self.xplane_addr).await?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Datarefs (RREF / DREF)
    // -----------------------------------------------------------------------

    /// Subscribe to a dataref, receiving updates `freq` times per second.
    ///
    /// `index` is a user-chosen identifier included in every
    /// [`Response::DatarefValues`] response, so you can match values back to
    /// your subscriptions.
    ///
    /// Append `[n]` to target a specific array element, e.g.
    /// `"sim/flightmodel/engine/POINT_thrust[0]"`.
    ///
    /// Call [`unsubscribe_dataref`] to stop updates.
    pub async fn subscribe_dataref(&self, freq: i32, index: i32, dataref: &str) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 4 + 4 + DATAREF_DIM);
        write_header(&mut buf, b"RREF");
        write_i32(&mut buf, freq);
        write_i32(&mut buf, index);
        write_str_zero(&mut buf, dataref, DATAREF_DIM, "dataref")?;
        self.send(buf).await
    }

    /// Stop receiving updates for the dataref identified by `index`.
    pub async fn unsubscribe_dataref(&self, index: i32) -> Result<()> {
        self.subscribe_dataref(0, index, "").await
    }

    /// Set a dataref to a float value.
    ///
    /// Integers and booleans in X-Plane are also sent as floats (e.g. `1.0`
    /// to enable a switch).
    pub async fn set_dataref(&self, dataref: &str, value: f32) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 4 + STR_DIM);
        write_header(&mut buf, b"DREF");
        write_f32(&mut buf, value);
        write_str_spaces(&mut buf, dataref, STR_DIM, "dataref")?;
        self.send(buf).await
    }

    // -----------------------------------------------------------------------
    // Commands (CMND)
    // -----------------------------------------------------------------------

    /// Execute an X-Plane command, e.g. `"sim/flight_controls/flaps_up"`.
    ///
    /// You can find command strings in X-Plane under
    /// Settings → Joystick → Buttons: Advanced.
    pub async fn send_command(&self, command: &str) -> Result<()> {
        let bytes = command.as_bytes();
        let mut buf = Vec::with_capacity(5 + bytes.len() + 1);
        write_header(&mut buf, b"CMND");
        buf.extend_from_slice(bytes);
        buf.push(0);
        self.send(buf).await
    }

    // -----------------------------------------------------------------------
    // Position streaming (RPOS)
    // -----------------------------------------------------------------------

    /// Request X-Plane to stream aircraft position at `freq` Hz.
    ///
    /// Responses arrive as [`Response::Position`]. Use [`stop_position`] or
    /// call this with `freq = 0` to stop.
    pub async fn request_position(&self, freq: u32) -> Result<()> {
        self.send_freq_cmd(b"RPOS", freq).await
    }

    /// Stop the aircraft position stream.
    pub async fn stop_position(&self) -> Result<()> {
        self.request_position(0).await
    }

    // -----------------------------------------------------------------------
    // Weather radar (RADR)
    // -----------------------------------------------------------------------

    /// Request X-Plane to stream weather radar data (`freq` points per frame).
    ///
    /// Responses arrive as [`Response::Radar`]. Use [`stop_radar`] to stop.
    pub async fn request_radar(&self, freq: u32) -> Result<()> {
        self.send_freq_cmd(b"RADR", freq).await
    }

    /// Stop the weather radar stream.
    pub async fn stop_radar(&self) -> Result<()> {
        self.request_radar(0).await
    }

    // -----------------------------------------------------------------------
    // Visual override (VEHX / VEHS)
    // -----------------------------------------------------------------------

    /// Drive X-Plane's visuals directly, overriding the flight model.
    ///
    /// `index` is the aircraft slot (0 = player aircraft, 1–19 = AI).
    pub async fn drive_visuals(
        &self,
        index: i32,
        lat: f64,
        lon: f64,
        ele: f64,
        heading: f32,
        pitch: f32,
        roll: f32,
    ) -> Result<()> {
        self.send_veh(b"VEHX", index, lat, lon, ele, heading, pitch, roll)
            .await
    }

    /// Move an aircraft to a position without overriding the flight model.
    pub async fn move_aircraft(
        &self,
        index: i32,
        lat: f64,
        lon: f64,
        ele: f64,
        heading: f32,
        pitch: f32,
        roll: f32,
    ) -> Result<()> {
        self.send_veh(b"VEHS", index, lat, lon, ele, heading, pitch, roll)
            .await
    }

    async fn send_veh(
        &self,
        tag: &[u8; 4],
        index: i32,
        lat: f64,
        lon: f64,
        ele: f64,
        heading: f32,
        pitch: f32,
        roll: f32,
    ) -> Result<()> {
        // Wire order: index (i32), lat (f64), lon (f64), ele (f64),
        //             heading (f32), pitch (f32), roll (f32).
        let mut buf = Vec::with_capacity(5 + 4 + 8 + 8 + 8 + 4 + 4 + 4);
        write_header(&mut buf, tag);
        write_i32(&mut buf, index);
        write_f64(&mut buf, lat);
        write_f64(&mut buf, lon);
        write_f64(&mut buf, ele);
        write_f32(&mut buf, heading);
        write_f32(&mut buf, pitch);
        write_f32(&mut buf, roll);
        self.send(buf).await
    }

    // -----------------------------------------------------------------------
    // Data output (DATA / DSEL / USEL)
    // -----------------------------------------------------------------------

    /// Send a DATA packet to X-Plane to set values directly.
    ///
    /// `index` corresponds to a row in X-Plane's Data Output screen.
    /// Use `-999.0` for values you want X-Plane to continue controlling.
    pub async fn send_data(&self, index: i32, values: [f32; 8]) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 4 + 8 * 4);
        write_header(&mut buf, b"DATA");
        write_i32(&mut buf, index);
        for v in values {
            write_f32(&mut buf, v);
        }
        self.send(buf).await
    }

    /// Ask X-Plane to start streaming the given data output indices (DSEL).
    ///
    /// Responses arrive as [`Response::Data`].
    pub async fn select_data(&self, indices: &[i32]) -> Result<()> {
        self.send_sel(b"DSEL", indices).await
    }

    /// Ask X-Plane to stop streaming the given data output indices (USEL).
    pub async fn unselect_data(&self, indices: &[i32]) -> Result<()> {
        self.send_sel(b"USEL", indices).await
    }

    async fn send_sel(&self, tag: &[u8; 4], indices: &[i32]) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + indices.len() * 4);
        write_header(&mut buf, tag);
        for &i in indices {
            write_i32(&mut buf, i);
        }
        self.send(buf).await
    }

    // -----------------------------------------------------------------------
    // Alerts (ALRT)
    // -----------------------------------------------------------------------

    /// Display an alert message in X-Plane (up to 4 lines, max 239 chars each).
    pub async fn alert(&self, line1: &str, line2: &str, line3: &str, line4: &str) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 4 * 240);
        write_header(&mut buf, b"ALRT");
        for (line, name) in [
            (line1, "line1"),
            (line2, "line2"),
            (line3, "line3"),
            (line4, "line4"),
        ] {
            write_str_zero(&mut buf, line, 240, name)?;
        }
        self.send(buf).await
    }

    // -----------------------------------------------------------------------
    // Failures (FAIL / RECO / RESE)
    // -----------------------------------------------------------------------

    /// Fail a system by its index string (e.g. `"0"` for the first failure
    /// listed in X-Plane's failure window).
    pub async fn fail_system(&self, index: &str) -> Result<()> {
        self.send_ascii_payload(b"FAIL", index).await
    }

    /// Recover a failed system by its index string.
    pub async fn recover_system(&self, index: &str) -> Result<()> {
        self.send_ascii_payload(b"RECO", index).await
    }

    async fn send_ascii_payload(&self, tag: &[u8; 4], s: &str) -> Result<()> {
        let bytes = s.as_bytes();
        let mut buf = Vec::with_capacity(5 + bytes.len() + 1);
        write_header(&mut buf, tag);
        buf.extend_from_slice(bytes);
        buf.push(0);
        self.send(buf).await
    }

    /// Recover all failed systems at once.
    pub async fn recover_all(&self) -> Result<()> {
        let mut buf = Vec::with_capacity(5);
        write_header(&mut buf, b"RESE");
        self.send(buf).await
    }

    // -----------------------------------------------------------------------
    // Fail / recover navaids (NFAL / NREC)
    // -----------------------------------------------------------------------

    /// Fail a navaid by its ID string.
    pub async fn fail_navaid(&self, navaid_id: &str) -> Result<()> {
        self.send_ascii_payload(b"NFAL", navaid_id).await
    }

    /// Recover a failed navaid by its ID string.
    pub async fn recover_navaid(&self, navaid_id: &str) -> Result<()> {
        self.send_ascii_payload(b"NREC", navaid_id).await
    }

    // -----------------------------------------------------------------------
    // Aircraft management (ACFN / PREL / ACPR)
    // -----------------------------------------------------------------------

    /// Load an aircraft by relative path.
    ///
    /// `index` is the aircraft slot (0 = player, 1–19 = AI aircraft).
    /// `live_index` is an internal index; pass `0` if unsure.
    pub async fn load_aircraft(&self, index: i32, path: &str, live_index: i32) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 4 + NET_STR_DIM + 2 + 4);
        write_header(&mut buf, b"ACFN");
        write_i32(&mut buf, index);
        write_str_zero(&mut buf, path, NET_STR_DIM, "path")?;
        buf.extend_from_slice(&[0u8; 2]); // pad
        write_i32(&mut buf, live_index);
        self.send(buf).await
    }

    /// Initialize an aircraft at a location without loading a new model.
    pub async fn place_aircraft(&self, cfg: &PlacementConfig<'_>) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 64);
        write_header(&mut buf, b"PREL");
        write_prel(&mut buf, cfg)?;
        self.send(buf).await
    }

    /// Load an aircraft and immediately initialize it at a location (ACPR).
    pub async fn load_and_place_aircraft(
        &self,
        index: i32,
        path: &str,
        live_index: i32,
        placement: &PlacementConfig<'_>,
    ) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 4 + NET_STR_DIM + 2 + 4 + 64);
        write_header(&mut buf, b"ACPR");
        write_i32(&mut buf, index);
        write_str_zero(&mut buf, path, NET_STR_DIM, "path")?;
        buf.extend_from_slice(&[0u8; 2]); // pad
        write_i32(&mut buf, live_index);
        write_prel(&mut buf, placement)?;
        self.send(buf).await
    }

    // -----------------------------------------------------------------------
    // Situation / movie (SIMO)
    // -----------------------------------------------------------------------

    /// Load or save a situation or movie file.
    pub async fn situation(&self, op: SituationOp, path: &str) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 4 + NET_STR_DIM + 2);
        write_header(&mut buf, b"SIMO");
        write_i32(&mut buf, i32::from(op));
        write_str_zero(&mut buf, path, NET_STR_DIM, "path")?;
        buf.extend_from_slice(&[0u8; 2]); // pad
        self.send(buf).await
    }

    // -----------------------------------------------------------------------
    // Sound (SOUN / LSND / SSND)
    // -----------------------------------------------------------------------

    /// Play a WAV file once. `freq` and `vol` are in the range 0.0–1.0.
    pub async fn play_sound(&self, freq: f32, vol: f32, path: &str) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 4 + 4 + STR_DIM);
        write_header(&mut buf, b"SOUN");
        write_f32(&mut buf, freq);
        write_f32(&mut buf, vol);
        write_str_zero(&mut buf, path, STR_DIM, "path")?;
        self.send(buf).await
    }

    /// Start a looping sound. `index` selects one of 5 simultaneous loops (0–4).
    pub async fn start_looping_sound(
        &self,
        index: i32,
        freq: f32,
        vol: f32,
        path: &str,
    ) -> Result<()> {
        self.send_loop_sound(b"LSND", index, freq, vol, path).await
    }

    /// Stop a looping sound started with [`start_looping_sound`].
    pub async fn stop_looping_sound(
        &self,
        index: i32,
        freq: f32,
        vol: f32,
        path: &str,
    ) -> Result<()> {
        self.send_loop_sound(b"SSND", index, freq, vol, path).await
    }

    async fn send_loop_sound(
        &self,
        tag: &[u8; 4],
        index: i32,
        freq: f32,
        vol: f32,
        path: &str,
    ) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 4 + 4 + 4 + STR_DIM);
        write_header(&mut buf, tag);
        write_i32(&mut buf, index);
        write_f32(&mut buf, freq);
        write_f32(&mut buf, vol);
        write_str_zero(&mut buf, path, STR_DIM, "path")?;
        self.send(buf).await
    }

    // -----------------------------------------------------------------------
    // Objects (OBJN / OBJL)
    // -----------------------------------------------------------------------

    /// Load a 3D object (OBJ7 format) into the given slot.
    pub async fn load_object(&self, index: i32, path: &str) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 4 + STR_DIM);
        write_header(&mut buf, b"OBJN");
        write_i32(&mut buf, index);
        write_str_zero(&mut buf, path, STR_DIM, "path")?;
        self.send(buf).await
    }

    /// Place a loaded 3D object at a world position.
    ///
    /// If `on_ground` is `true`, set `ele` to `0.0`; X-Plane will snap the
    /// object to the terrain.
    pub async fn place_object(
        &self,
        index: i32,
        lat: f64,
        lon: f64,
        ele: f64,
        heading: f32,
        pitch: f32,
        roll: f32,
        on_ground: bool,
        smoke_size: f32,
    ) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 4 + 4 + 8 * 3 + 4 * 3 + 4 + 4 + 4);
        write_header(&mut buf, b"OBJL");
        write_i32(&mut buf, index);
        buf.extend_from_slice(&[0u8; 4]); // pad1
        write_f64(&mut buf, lat);
        write_f64(&mut buf, lon);
        write_f64(&mut buf, ele);
        write_f32(&mut buf, heading);
        write_f32(&mut buf, pitch);
        write_f32(&mut buf, roll);
        write_i32(&mut buf, on_ground as i32);
        write_f32(&mut buf, smoke_size);
        buf.extend_from_slice(&[0u8; 4]); // pad2
        self.send(buf).await
    }

    // -----------------------------------------------------------------------
    // Network configuration (ISE4 / ISE6)
    // -----------------------------------------------------------------------

    /// Configure a UDP output target (IPv4).
    ///
    /// `index` selects which function to configure (see X-Plane ISE4 documentation).
    /// `ip` is a dotted-decimal string (e.g. `"192.168.1.5"`).
    /// `port` is the port number as a string (e.g. `"49000"`).
    pub async fn set_network_v4(
        &self,
        index: i32,
        ip: &str,
        port: &str,
        enabled: bool,
    ) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 4 + 16 + 8 + 4);
        write_header(&mut buf, b"ISE4");
        write_i32(&mut buf, index);
        write_str_zero(&mut buf, ip, 16, "ip")?;
        write_str_zero(&mut buf, port, 8, "port")?;
        write_i32(&mut buf, enabled as i32);
        self.send(buf).await
    }

    /// Configure a UDP output target (IPv6).
    pub async fn set_network_v6(
        &self,
        index: i32,
        ip: &str,
        port: &str,
        enabled: bool,
    ) -> Result<()> {
        let mut buf = Vec::with_capacity(5 + 4 + 65 + 6 + 1 + 4);
        write_header(&mut buf, b"ISE6");
        write_i32(&mut buf, index);
        write_str_zero(&mut buf, ip, 65, "ip")?;
        write_str_zero(&mut buf, port, 6, "port")?;
        buf.push(0); // pad1
        write_i32(&mut buf, enabled as i32);
        self.send(buf).await
    }

    // -----------------------------------------------------------------------
    // Control (QUIT / SHUT)
    // -----------------------------------------------------------------------

    /// Request X-Plane to quit gracefully.
    pub async fn quit(&self) -> Result<()> {
        let mut buf = Vec::with_capacity(5);
        write_header(&mut buf, b"QUIT");
        self.send(buf).await
    }

    /// Shut down X-Plane immediately.
    pub async fn shutdown(&self) -> Result<()> {
        let mut buf = Vec::with_capacity(5);
        write_header(&mut buf, b"SHUT");
        self.send(buf).await
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write_prel(buf: &mut Vec<u8>, cfg: &PlacementConfig<'_>) -> Result<()> {
    write_i32(buf, i32::from(cfg.start_type));
    write_i32(buf, cfg.aircraft_index);
    write_str_zero(buf, cfg.airport_id, 8, "airport_id")?;
    write_i32(buf, cfg.runway_index);
    write_i32(buf, cfg.runway_direction);
    write_f64(buf, cfg.latitude);
    write_f64(buf, cfg.longitude);
    write_f64(buf, cfg.elevation);
    write_f64(buf, cfg.heading);
    write_f64(buf, cfg.speed);
    Ok(())
}
