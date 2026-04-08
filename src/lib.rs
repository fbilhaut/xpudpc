//! # xpudpc
//!
//! A Rust client for X-Plane's built-in UDP protocol.
//!
//! Communicate with a running X-Plane instance to read and set datarefs,
//! stream aircraft position, execute commands, manage aircraft and objects,
//! and more — all over UDP with no plugins required.
//!
//! ## Connecting
//!
//! If you know X-Plane's address, connect directly:
//!
//! ```no_run
//! let client = xpudpc::XPlaneClient::connect("127.0.0.1:49000")?;
//! # Ok::<(), xpudpc::Error>(())
//! ```
//!
//! Or auto-discover X-Plane on the local network using the multicast beacon:
//!
//! ```no_run
//! use std::time::Duration;
//!
//! let beacon = xpudpc::find_xplane(Some(Duration::from_secs(10)))?;
//! let client = xpudpc::XPlaneClient::connect(
//!     format!("{}:{}", beacon.ip, beacon.port)
//! )?;
//! # Ok::<(), xpudpc::Error>(())
//! ```
//!
//! ## Reading data
//!
//! Subscribe to datarefs with [`XPlaneClient::subscribe_dataref`] or request
//! an aircraft position stream with [`XPlaneClient::request_position`], then
//! loop on [`XPlaneClient::recv`]:
//!
//! ```no_run
//! use xpudpc::{XPlaneClient, Response};
//! use std::time::Duration;
//!
//! let client = XPlaneClient::connect("127.0.0.1:49000")?;
//! client.set_recv_timeout(Some(Duration::from_secs(1)))?;
//!
//! // Subscribe to indicated airspeed, using index 0 as our identifier.
//! client.subscribe_dataref(10, 0, "sim/cockpit2/gauges/indicators/airspeed_kts_pilot")?;
//!
//! // Stream position at 30 Hz.
//! client.request_position(30.0)?;
//!
//! loop {
//!     match client.recv()? {
//!         Response::DatarefValues(refs) => {
//!             for r in refs {
//!                 println!("dataref {}: {:.1}", r.index, r.value);
//!             }
//!         }
//!         Response::Position(pos) => {
//!             println!("lat={:.4} lon={:.4} alt={:.0}m", pos.latitude, pos.longitude, pos.elevation);
//!         }
//!         _ => {}
//!     }
//! }
//! # Ok::<(), xpudpc::Error>(())
//! ```
//!
//! ## Writing data
//!
//! ```no_run
//! let client = xpudpc::XPlaneClient::connect("127.0.0.1:49000")?;
//!
//! // Set a dataref.
//! client.set_dataref("sim/cockpit/switches/anti_ice_surf_heat_left", 1.0)?;
//!
//! // Execute a command.
//! client.send_command("sim/flight_controls/flaps_up")?;
//! # Ok::<(), xpudpc::Error>(())
//! ```

mod codec;

pub mod beacon;
pub mod client;
pub mod error;
pub mod response;
pub mod types;

pub use beacon::find_xplane;
pub use client::XPlaneClient;
pub use error::{Error, Result};
pub use response::Response;
pub use types::{
    beacon::BeaconInfo, data::DataOutput, dataref::DatarefValue, placement::PlacementConfig,
    placement::StartType, position::AircraftPosition, radar::RadarPoint, situation::SituationOp,
};
