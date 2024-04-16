use crate::commands::{BINARY_PATH_STR, FIRMWARE_PATCHED_STR};

use super::install::is_installed;
use std::{
    fs::{copy, File, OpenOptions},
    io::prelude::*,
    path::PathBuf,
    process::Command,
};
use uname_rs;

pub fn apply() -> Result<(), String> {
    println!("Applying firmware patch. WiFi will be disabled!");

    // check whether nexmon csi is installed
    if !is_installed() {
        return Err(format!(
            "Nexmon CSI is not installed! Run 'sudo cspi install' before using this command."
        ));
    }

    // Check whether firmware is already patched
    let firmware_patched_path = PathBuf::from(FIRMWARE_PATCHED_STR);
    if firmware_patched_path.exists() {
        return Err(format!("Firmware is already patched!"));
    }

    // Get system information
    let release = uname_rs::Uname::new()
        .map_err(|_| "Could not get system information")?
        .release;

    // disable wpa_supplicant
    println!("Disabling wpa_supplicant...");
    let mut dhcpcd_file = OpenOptions::new()
        .append(true)
        .open("/etc/dhcpcd.conf")
        .map_err(|err| format!("Cannot open dhcpcd.conf. Error: {}", err))?;

    writeln!(
        dhcpcd_file,
        "\ndenyinterfaces wlan0\ninterface wlan0\n\tnohook wpa_supplicant"
    )
    .map_err(|err| format!("Cannot block wpa_supplicant in dhcpcd.conf. Error: {}", err))?;

    Command::new("killall")
        .arg("wpa_supplicant")
        .status()
        .map_err(|err| format!("killall wpa_supplicant error: {}", err))?;

    Command::new("systemctl")
        .args(["disable", "--now", "wpa_supplicant"])
        .status()
        .map_err(|err| format!("systemctl disable --now wpa_supplicant error: {}", err))?;

    // Apply firmware patch
    println!("Applying firmware patch...");

    // prepare paths
    let binary_path = PathBuf::from(BINARY_PATH_STR.to_owned() + &release);
    let ko_path = String::from_utf8(
        Command::new("modinfo")
            .args(["brcmfmac", "-n"])
            .output()
            .map_err(|err| format!("Error running 'modinfo brcmfmac -n': {}", err))?
            .stdout,
    )
    .map_err(|err| {
        format!(
            "Could not read path returned by 'brcmfmac modinfo -n'. Error: {}",
            err
        )
    })?;

    // Patch brcmfmac43455-sdio.bin
    let mut sdio_patch_path = binary_path.clone();
    sdio_patch_path.push("patched/brcmfmac43455-sdio.bin");
    copy(
        &sdio_patch_path,
        "/lib/firmware/brcm/brcmfmac43455-sdio.bin",
    )
    .map_err(|err| format!("Cannot patch brcmfmac43455-sdio.bin. Error: {}", err))?;

    // Patch brcmfmac.ko
    let mut ko_patch_path = binary_path.clone();
    ko_patch_path.push("patched/brcmfmac.ko");
    copy(&ko_patch_path, ko_path)
        .map_err(|err| format!("Cannot patch brcmfmac.ko. Error: {}", err))?;

    // Update kernel modules
    println!("Updating kernel modules...");
    Command::new("rmmod")
        .arg("brcmfmac")
        .status()
        .map_err(|err| format!("rmmod error: {}", err))?;

    Command::new("modprobe")
        .arg("brcmutil")
        .status()
        .map_err(|err| format!("modprobe error: {}", err))?;

    Command::new("insmod")
        .arg(
            ko_patch_path
                .to_str()
                .expect("Could not convert path to string"),
        )
        .status()
        .map_err(|err| format!("insmod error: {}", err))?;

    // generate modules.dep and map files
    println!("Generating modules.dep and map files... (This may take a few seconds)");
    Command::new("depmod")
        .arg("-a")
        .status()
        .map_err(|err| format!("depmod error: {}", err))?;

    // Remember patch state
    File::create(FIRMWARE_PATCHED_STR)
        .map_err(|err| format!("Could not save patch state. Error: {}", err))?;

    println!("Applied patch successfully!");

    Ok(())
}

pub fn is_applied() -> bool {
    let is_patched_path = PathBuf::from(FIRMWARE_PATCHED_STR);

    // Check if nexmon_csi has already been installed
    is_patched_path.exists()
}