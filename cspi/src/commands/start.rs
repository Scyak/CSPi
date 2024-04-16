use std::{fs::File, process::Command};

use crate::commands::{
    apply::is_applied, decode::launch_decoder, running::is_running, NEXMON_RUNNING_STR,
};

pub fn start(channel: &u32, bandwidth: &u32, maclist: &String) -> Result<(), String> {
    println!("Starting CSI collection...");

    // check whether patch has been applied
    if !is_applied() {
        return Err(String::from("You must apply the firmware patch using cspi apply before starting or stopping collection!"));
    }

    // check whether CSI collection is already running
    if is_running() {
        return Err(String::from("CSI collection is already running"));
    }

    let channel_bandwidth = format!("{}/{}", channel, bandwidth);
    let mut arglist = vec!["-C", "1", "-N", "1", "-c", &channel_bandwidth];
    if !maclist.is_empty() {
        arglist.extend(["-m", maclist]);
    }

    // create parameter string
    let parameters = String::from_utf8(
        Command::new("mcp")
            .args(arglist)
            .output()
            .map_err(|err| format!("Could not create CSI parameter string. Error: {}", err))?
            .stdout,
    )
    .map_err(|_| "Could not parse string returned by mcp")?;

    // set up nexmon CSI using nexutil
    Command::new("ifconfig")
        .args(["wlan0", "up"])
        .status()
        .map_err(|err| format!("Error running ifconfig wlan0 up: {}", err))?;
    Command::new("nexutil")
        .args(["-Iwlan0", "-s500", "-b", "-l34"])
        .arg(format!("-v{}", parameters))
        .status()
        .map_err(|err| format!("Error using nexutil to set up nexmon: {}", err))?;

    // create mon0 if it doesn't exist
    let monitor_info = String::from_utf8(
        Command::new("ip")
            .args(["link", "show"])
            .output()
            .map_err(|err| {
                format!(
                    "Could not obtain information from ip link show. Error: {}",
                    err
                )
            })?
            .stdout,
    )
    .map_err(|_| "Could not parse string returned by ip link show")?;

    if !monitor_info.contains("mon0") {
        Command::new("iw")
            .args([
                "dev",
                "wlan0",
                "interface",
                "add",
                "mon0",
                "type",
                "monitor",
            ])
            .status()
            .map_err(|err| format!("Error adding mon0: {}", err))?;
    }

    // set up mon0
    Command::new("ifconfig")
        .args(["mon0", "up"])
        .status()
        .map_err(|err| format!("Error enabling mon0: {}", err))?;

    // Launch decoder in background if not running
    let _ = launch_decoder();

    // Remember running state
    File::create(NEXMON_RUNNING_STR)
        .map_err(|err| format!("Could not save running state. Error: {}", err))?;

    println!("CSI collection is running.\nCSI in nexmon format is available on port 5500.\nCSI in protobuf format is available on port 4400.");

    Ok(())
}
