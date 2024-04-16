use std::{process::Command, fs::remove_file};

use crate::commands::{apply::is_applied, NEXMON_RUNNING_STR, running::is_running};

pub fn stop() -> Result<(), String> {
    println!("Stopping CSI collection...");

    // check whether patch has been applied
    if !is_applied() {
        return Err(String::from("You must apply the firmware patch using cspi apply before starting or stopping collection!"));
    }

    // check whether CSI collection is running
    if !is_running() {
        return Err(String::from("CSI collection is not running, cannot stop"));
    }

    // create parameter string
    let parameters = String::from_utf8(
        Command::new("mcp")
            .args(["-e", "0"])
            .output()
            .map_err(|err| format!("Could not create CSI parameter string. Error: {}", err))?
            .stdout,
    )
    .map_err(|_| "Could not parse string returned by mcp")?;

    // stop nexmon CSI using nexutil
    Command::new("nexutil")
        .args(["-Iwlan0", "-s500", "-b", "-l34"])
        .arg(format!("-v{}", parameters))
        .status()
        .map_err(|err| format!("Error using nexutil to set up nexmon: {}", err))?;

    // Remember running state
    remove_file(NEXMON_RUNNING_STR)
        .map_err(|err| format!("Could not save running state. Error: {}", err))?;

    println!("CSI collection has been stopped successfully.");

    Ok(())
}
