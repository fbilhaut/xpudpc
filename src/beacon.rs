use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::UdpSocket;
use crate::codec::reader::Reader;
use crate::error::{Error, Result};
use crate::types::beacon::BeaconInfo;

const BEACON_MULTICAST: Ipv4Addr = Ipv4Addr::new(239, 255, 1, 1);
const BEACON_PORT: u16 = 49707;
const RECV_BUF: usize = 1024;

/// Listen for the X-Plane beacon and return the first X-Plane instance found.
///
/// X-Plane broadcasts a BECN packet to multicast group `239.255.1.1:49707`.
/// This function awaits until a beacon is received or `timeout` expires.
///
/// Only packets with `application_host_id == 1` (X-Plane, not PlaneMaker) are returned.
///
///
/// # Example
///
/// ```no_run
/// use xpudpc::find_xplane;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> xpudpc::Result<()> {
///     let beacon = find_xplane(Some(Duration::from_secs(10))).await?;
///     println!("Found X-Plane {} at {}:{}", beacon.version_number, beacon.ip, beacon.port);
///
///     let client = xpudpc::XPlaneClient::connect(
///         format!("{}:{}", beacon.ip, beacon.port)
///     ).await?;
///     Ok(())
/// }
/// ```
pub async fn find_xplane(timeout: Option<Duration>) -> Result<BeaconInfo> {
    match timeout {
        Some(d) => tokio::time::timeout(d, find_xplane_inner())
            .await
            .map_err(|_| {
                Error::Io(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "X-Plane not found within timeout",
                ))
            })?,
        None => find_xplane_inner().await,
    }
}

async fn find_xplane_inner() -> Result<BeaconInfo> {
    let socket = beacon_socket()?;
    socket.join_multicast_v4(BEACON_MULTICAST, Ipv4Addr::UNSPECIFIED)?;

    let mut buf = [0u8; RECV_BUF];
    loop {
        let (n, src) = socket.recv_from(&mut buf).await?;
        if let Ok(info) = parse_beacon(&buf[..n], src) {
            if info.application_host_id == 1 {
                return Ok(info);
            }
        }
    }
}

/// Create a UDP socket bound to the beacon port with SO_REUSEPORT (Unix) /
/// SO_REUSEADDR (Windows) so that multiple processes on the same machine
/// can all receive the X-Plane multicast beacon simultaneously.
fn beacon_socket() -> io::Result<UdpSocket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    socket.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, BEACON_PORT)).into())?;
    socket.set_nonblocking(true)?;
    UdpSocket::from_std(socket.into())
}

fn parse_beacon(buf: &[u8], src: SocketAddr) -> Result<BeaconInfo> {
    // minimum = 5-byte header + 16 bytes of mandatory fields.
    if buf.len() < 21 {
        return Err(Error::InvalidResponse("beacon packet too short".into()));
    }

    let tag: [u8; 4] = buf[..4].try_into().unwrap();
    if &tag != b"BECN" {
        return Err(Error::UnknownTag(tag));
    }

    let mut r = Reader::new(&buf[5..]);

    let beacon_major_version = r.read_u8()?;
    let beacon_minor_version = r.read_u8()?;
    let application_host_id = r.read_i32()?;
    let version_number = r.read_i32()?;
    let role = r.read_u32()?;
    let port = r.read_u16()?;

    // X-Plane 12 may send a truncated beacon (23 bytes total) that omits
    // computer_name. Only read it when the full 500-byte field is present.
    let computer_name = if r.remaining() >= 500 {
        r.read_str(500)?
    } else {
        String::new()
    };
    let raknet_port = if r.remaining() >= 2 { r.read_u16()? } else { 0 };

    Ok(BeaconInfo {
        ip: src.ip(),
        port,
        beacon_major_version,
        beacon_minor_version,
        application_host_id,
        version_number,
        role,
        computer_name,
        raknet_port,
    })
}
