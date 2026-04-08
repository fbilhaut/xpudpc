use std::time::Duration;
use xpudpc::{Response, XPlaneClient, find_xplane};

#[tokio::main]
async fn main() -> xpudpc::Result<()> {
    println!("Waiting for X-Plane beacon...");
    let beacon = find_xplane(Some(Duration::from_secs(30))).await?;
    println!(
        "Found X-Plane v{} at {}:{}\n",
        beacon.version_number, beacon.ip, beacon.port
    );

    let client = XPlaneClient::connect((beacon.ip, beacon.port)).await?;

    client.request_position(1).await?;
    println!("Streaming position at 1 Hz. Press Ctrl-C to stop.\n");

    loop {
        match client.recv_timeout(Duration::from_secs(3)).await {
            Ok(Response::Position(pos)) => {
                println!(
                    "lat={:>10.5}  lon={:>11.5}  alt={:>8.1}m  \
                     hdg={:>6.1}°  pitch={:>6.1}°  roll={:>6.1}°",
                    pos.latitude, pos.longitude, pos.elevation, pos.heading, pos.pitch, pos.roll,
                );
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
        }
    }

    client.stop_position().await?;
    Ok(())
}
