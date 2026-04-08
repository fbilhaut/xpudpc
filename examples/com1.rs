use std::time::Duration;
use xpudpc::{Response, XPlaneClient};

const IDX_FREQ: i32 = 0;
const IDX_RX: i32 = 1;
const IDX_TX: i32 = 2;
const IDX_PWR: i32 = 3;
const IDX_BUS_VOLTS: i32 = 4;
const IDX_AVIONICS: i32 = 5;

#[tokio::main]
async fn main() -> xpudpc::Result<()> {
    let client = XPlaneClient::connect("192.168.1.25:49000").await?;

    client
        .subscribe_dataref(
            1,
            IDX_FREQ,
            "sim/cockpit2/radios/actuators/com1_frequency_hz_833",
        )
        .await?;
    client
        .subscribe_dataref(
            1,
            IDX_RX,
            "sim/cockpit2/radios/actuators/audio_selection_com1",
        )
        .await?;
    client
        .subscribe_dataref(1, IDX_TX, "sim/cockpit/switches/audio_panel_out")
        .await?;
    client
        .subscribe_dataref(1, IDX_PWR, "sim/cockpit2/radios/actuators/com1_power")
        .await?;
    client
        .subscribe_dataref(1, IDX_BUS_VOLTS, "sim/cockpit2/electrical/bus_volts[0]")
        .await?;
    client
        .subscribe_dataref(1, IDX_AVIONICS, "sim/cockpit2/switches/avionics_power_on")
        .await?;

    println!("Streaming COM1 data at 1 Hz. Press Ctrl-C to stop.\n");

    let mut freq_mhz: Option<f32> = None;
    let mut rx: Option<bool> = None;
    let mut tx: Option<bool> = None;
    let mut power: Option<bool> = None;
    let mut volts: Option<f32> = None;
    let mut avionics: Option<bool> = None;

    loop {
        match client.recv_timeout(Duration::from_secs(3)).await {
            Ok(Response::DatarefValues(refs)) => {
                for r in refs {
                    match r.index {
                        IDX_FREQ => freq_mhz = Some(r.value / 1_000.0),
                        IDX_RX => rx = Some(r.value != 0.0),
                        IDX_TX => tx = Some(r.value == 6.0),
                        IDX_PWR => power = Some(r.value != 0.0),
                        IDX_BUS_VOLTS => volts = Some(r.value),
                        IDX_AVIONICS => avionics = Some(r.value != 0.0),
                        _ => {}
                    }
                }

                if let (Some(freq), Some(rx), Some(tx), Some(power), Some(avionics), Some(volts)) =
                    (freq_mhz, rx, tx, power, avionics, volts)
                {
                    let powered = power && avionics && volts > 0.0;
                    println!(
                        "COM1  freq={} MHz  RX={}  TX={}  PWR={}",
                        freq,
                        if rx { "on " } else { "off" },
                        if tx { "on " } else { "off" },
                        if powered { "on " } else { "off" },
                    );
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
        }
    }

    client.unsubscribe_dataref(IDX_FREQ).await?;
    client.unsubscribe_dataref(IDX_RX).await?;
    client.unsubscribe_dataref(IDX_TX).await?;
    client.unsubscribe_dataref(IDX_PWR).await?;
    client.unsubscribe_dataref(IDX_BUS_VOLTS).await?;
    client.unsubscribe_dataref(IDX_AVIONICS).await?;
    
    Ok(())
}
