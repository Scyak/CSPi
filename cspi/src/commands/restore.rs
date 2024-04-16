use crate::commands::{running::is_running, stop::stop, BINARY_PATH_STR, FIRMWARE_PATCHED_STR};

use super::install::is_installed;
use std::{
    fs::{copy, remove_file, OpenOptions, File},
    io::prelude::*,
    path::PathBuf,
    process::Command,
};
use uname_rs;

pub fn restore() -> Result<(), String> {
    println!("Restoring original WiFi firmware and re-enabling WiFi...");

    // check whether nexmon csi is installed
    if !is_installed() {
        return Err(format!(
            "Nexmon CSI is not installed! Run 'sudo cspi install' before using this command."
        ));
    }

    // Check whether firmware is already the original
    let firmware_patched_path = PathBuf::from(FIRMWARE_PATCHED_STR);
    if !firmware_patched_path.exists() {
        return Err(format!(
            "Firmware is already the original! Nothing to restore."
        ));
    }

    // Get system information
    let release = uname_rs::Uname::new()
        .map_err(|_| "Could not get system information")?
        .release;

    // Stop CSI collection
    if is_running() {
        stop()?;
    }

    // remove mon0
    println!("Removing mon0");
    Command::new("ip")
        .args(["link", "set", "mon0", "down"])
        .status()
        .map_err(|err| format!("Error disabling mon0: {}", err))?;
    Command::new("iw")
        .args(["dev", "mon0", "del"])
        .status()
        .map_err(|err| format!("Error removing mon0: {}", err))?;

    // Restart wlan0
    println!("Restarting wlan0...");

    Command::new("ip")
        .args(["link", "set", "dev", "wlan0", "down"])
        .status()
        .map_err(|err| format!("Could not set wlan0 down. Error: {}", err))?;

    Command::new("ip")
        .args(["link", "set", "dev", "wlan0", "up"])
        .status()
        .map_err(|err| format!("Could not set wlan0 up. Error: {}", err))?;

    // Unblock wpa_supplicant
    println!("Unblocking wpa_supplicant...");
    let mut dhcpcd_read = OpenOptions::new()
        .read(true)
        .open("/etc/dhcpcd.conf")
        .map_err(|err| format!("Could not open dhcpcd.conf. Error: {}", err))?;

    let mut dhcpcd_contents = String::new();
    dhcpcd_read
        .read_to_string(&mut dhcpcd_contents)
        .map_err(|err| format!("Could not read from dhcpcd.conf. Error: {}", err))?;
    let new_dhcpcd = dhcpcd_contents.replace(
        "\ndenyinterfaces wlan0\ninterface wlan0\n\tnohook wpa_supplicant\n",
        "",
    );

    drop(dhcpcd_read);

    let mut dhcpcd_write = File::create("/etc/dhcpcd.conf").map_err(|e| format!("Cannot open /etc/dhcpcd.conf. Error: {}", e))?;
    write!(dhcpcd_write, "{}", new_dhcpcd)
        .map_err(|err| format!("Could not write to dhcpcd.conf. Error: {}", err))?;

    // Restore original firmware
    // prepare paths
    let binary_path = PathBuf::from(BINARY_PATH_STR.to_owned() + &release);
    let ko_path = String::from_utf8(
        Command::new("modinfo")
            .args(["brcmfmac", "-n"])
            .output()
            .expect("Failed to read brcmfmac modinfo")
            .stdout,
    )
    .map_err(|err| {
        format!(
            "Could not read path returned by 'brcmfmac modinfo -n'. Error: {}",
            err
        )
    })?;

    // Restore brcmfmac43455-sdio.bin
    println!("Restoring original firmware files...");
    let mut sdio_patch_path = binary_path.clone();
    sdio_patch_path.push("original/brcmfmac43455-sdio.bin");
    copy(
        &sdio_patch_path,
        "/lib/firmware/brcm/brcmfmac43455-sdio.bin",
    )
    .map_err(|err| format!("Cannot restore brcmfmac43455-sdio.bin. Error: {}", err))?;

    // Patch brcmfmac.ko
    let mut ko_patch_path = binary_path.clone();
    ko_patch_path.push("original/brcmfmac.ko");
    copy(&ko_patch_path, ko_path)
        .map_err(|err| format!("Cannot restore brcmfmac.ko. Error: {}", err))?;

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

    // Re-enable wpa_supplicant on wlan0
    println!("Re-enabling wpa_supplicant...");
    Command::new("systemctl")
        .args(["enable", "wpa_supplicant"])
        .status()
        .map_err(|err| {
            format!(
                "Could not re-enable wpa_supplicant (systemctl). Error: {}",
                err
            )
        })?;

    Command::new("wpa_supplicant")
        .args([
            "-B",
            "-c",
            "/etc/wpa_supplicant/wpa_supplicant.conf",
            "-i",
            "wlan0",
        ])
        .status()
        .map_err(|err| format!("Could not start wpa_supplicant. Error: {}", err))?;

    Command::new("dhcpcd")
        .arg("wlan0")
        .status()
        .map_err(|err| format!("Error running dhcpcd wlan0: {}", err))?;

    // Restart wlan0
    println!("Restarting wlan0...");

    Command::new("ip")
        .args(["link", "set", "dev", "wlan0", "down"])
        .status()
        .map_err(|err| format!("Could not set wlan0 down. Error: {}", err))?;

    Command::new("ip")
        .args(["link", "set", "dev", "wlan0", "up"])
        .status()
        .map_err(|err| format!("Could not set wlan0 up. Error: {}", err))?;

    // Restart dhcpcd
    Command::new("service")
        .args(["dhcpcd", "restart"])
        .status()
        .map_err(|err| format!("Could not restart dhcpcd. Error: {}", err))?;

    // Remember patch state
    remove_file(FIRMWARE_PATCHED_STR)
        .map_err(|err| format!("Could not save patch state. Error: {}", err))?;

    println!("Original firmware restored successfully!");
    println!("Note: It may take a few seconds to reconnect to previous WiFi network.");

    Ok(())
}
