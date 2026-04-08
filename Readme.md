# xpudpc

A Rust async client for [X-Plane](https://www.x-plane.com/)'s built-in UDP protocol.

Communicate with a running X-Plane instance to read and set datarefs, stream
aircraft position, execute commands, manage aircraft and objects, etc. 
All over UDP with no plugins required.

## Features

- Read datarefs at a configurable frequency
- Set datarefs
- Execute commands
- Stream aircraft position and weather radar 
- Drive X-Plane visuals from an external flight model
- Send data output values
- Manage aircraft: load, place, load-and-place
- Auto-discover X-Plane on the local network via UDP beacon
- Fully async

## Installation

```toml
[dependencies]
xpudpc = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## X-Plane setup

In Settings → Network, make sure *Accept incoming connections* is enabled (required on X-Plane 12).

## Quick start

### Connect by address

```rust
use std::time::Duration;
use xpudpc::{Response, XPlaneClient};

#[tokio::main]
async fn main() -> xpudpc::Result<()> {
    let client = XPlaneClient::connect("192.168.1.10:49000").await?;

    // Subscribe to indicated airspeed — index 0 is our local identifier
    client.subscribe_dataref(10, 0, "sim/cockpit2/gauges/indicators/airspeed_kts_pilot").await?;

    loop {
        if let Ok(Response::DatarefValues(refs)) = client.recv_timeout(Duration::from_secs(1)).await {
            for r in refs {
                println!("airspeed: {:.1} kts", r.value);
            }
        }
    }
}
```

### Auto-discover via beacon

X-Plane broadcasts a UDP beacon on the local network. Use `find_xplane` to
locate it automatically instead of hard-coding an address:

```rust
use std::time::Duration;
use xpudpc::{find_xplane, XPlaneClient};

#[tokio::main]
async fn main() -> xpudpc::Result<()> {
    let beacon = find_xplane(Some(Duration::from_secs(30))).await?;
    println!("Found X-Plane v{} at {}:{}", beacon.version_number, beacon.ip, beacon.port);

    let client = XPlaneClient::connect((beacon.ip, beacon.port)).await?;
    // ...
    Ok(())
}
```

## Usage

### Receiving data

All incoming data arrives through `recv()` (or `recv_timeout()`) as a
`Response` enum. Set up one or more streams before entering the receive loop:

```rust
// Position at 30 Hz
client.request_position(30).await?;

// Two datarefs, each with a local index for identification
client.subscribe_dataref(10, 0, "sim/cockpit2/gauges/indicators/airspeed_kts_pilot").await?;
client.subscribe_dataref(5,  1, "sim/cockpit2/gauges/indicators/altitude_ft_pilot").await?;

loop {
    match client.recv_timeout(Duration::from_secs(3)).await? {
        Response::Position(pos) => {
            println!("lat={:.5} lon={:.5} alt={:.0}m hdg={:.1}°",
                pos.latitude, pos.longitude, pos.elevation, pos.heading);
        }
        Response::DatarefValues(refs) => {
            for r in refs {
                match r.index {
                    0 => println!("airspeed : {:.1} kts", r.value),
                    1 => println!("altitude : {:.0} ft",  r.value),
                    _ => {}
                }
            }
        }
        _ => {}
    }
}
```

Unsubscribe or stop streams when no longer needed:

```rust
client.unsubscribe_dataref(0).await?;
client.stop_position().await?;
```

### Setting datarefs

All X-Plane datarefs accept float values; pass `1.0` or `0.0` for boolean
switches:

```rust
client.set_dataref("sim/cockpit/switches/anti_ice_surf_heat_left", 1.0).await?;
```

### Executing commands

```rust
client.send_command("sim/flight_controls/flaps_up").await?;
client.send_command("sim/flight_controls/landing_gear_toggle").await?;
```

Command strings can be found in X-Plane under
Settings → Joystick → Buttons: Advanced.

### Placing an aircraft

```rust
use xpudpc::{PlacementConfig, StartType};

client.place_aircraft(&PlacementConfig {
    start_type: StartType::SpecifyLatLonEle,
    latitude:   37.6189,
    longitude: -122.3750,
    elevation:  4.0,
    heading:    280.0,
    ..Default::default()
}).await?;
```

### Driving visuals from an external flight model

```rust
// Override X-Plane's flight model entirely (VEHX)
client.drive_visuals(0, lat, lon, ele, heading, pitch, roll).await?;

// Or move the aircraft once without overriding the flight model (VEHS)
client.move_aircraft(0, lat, lon, ele, heading, pitch, roll).await?;
```

### Failures

```rust
client.fail_system("0").await?;   // fail the first system in the failure list
client.recover_system("0").await?;
client.recover_all().await?;      // reset everything at once
```

## Response types

| Variant | Triggered by |
|---|---|
| `Response::Position(AircraftPosition)` | `request_position` |
| `Response::Radar(Vec<RadarPoint>)` | `request_radar` |
| `Response::DatarefValues(Vec<DatarefValue>)` | `subscribe_dataref` |
| `Response::Data(DataOutput)` | `select_data` or X-Plane data output screen |

## Examples

```sh
# Stream position at 1 Hz (hard-coded address)
cargo run --example position

# Same, but auto-discover X-Plane via beacon
cargo run --example position-beacon
```

## Protocol notes

- X-Plane listens on **port 49000** by default.
- Replies are sent back to whichever IP/port the request came from.
- All values are **little-endian**. 
- Frequencies for RPOS and RADR are transmitted as null-terminated ASCII strings.
- The UDP beacon is multicast on group `239.255.1.1:49707`.
