/// Information about an X-Plane instance discovered via the UDP beacon.
///
/// Returned by [`crate::find_xplane`].
#[derive(Debug, Clone)]
pub struct BeaconInfo {
    /// IP address of the X-Plane instance.
    pub ip: std::net::IpAddr,
    /// UDP port X-Plane is listening on (usually 49000).
    pub port: u16,
    /// Beacon protocol major version.
    pub beacon_major_version: u8,
    /// Beacon protocol minor version.
    pub beacon_minor_version: u8,
    /// Application host ID: 1 = X-Plane, 2 = PlaneMaker.
    pub application_host_id: i32,
    /// X-Plane version number (e.g. `120000` for 12.00, `115500` for 11.55).
    pub version_number: i32,
    /// Role: 1 = master, 2 = external visual, 3 = IOS.
    pub role: u32,
    /// Hostname of the X-Plane computer.
    pub computer_name: String,
    /// Port for the RakNet client (added in beacon minor version 2).
    pub raknet_port: u16,
}
