use crate::commands::BINARY_PATH_STR;

use std::fs::{create_dir_all, remove_file, File};
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;
use tar::Archive;
use uname_rs;
use xz2::read::XzDecoder;

const NEXMON_INSTALLED_STR: &str = "/home/pi/.cspi/nexmon-installed";

/// If not yet installed, downloads nexmon CSI precompiled binaries from nexmonster and installs them
pub fn install(force: &bool) -> Result<(), String> {
    println!("Installing nexmon csi...");

    // Get system information
    let release = uname_rs::Uname::new()
        .map_err(|_| "Could not get system information")?
        .release;

    // Prepare installation
    create_dir_all(PathBuf::from(BINARY_PATH_STR))
        .map_err(|_| "Could not create install directory")?;

    // Check if nexmon_csi has already been installed
    if !force {
        if is_installed() {
            return Err(format!("Already installed"));
        }
    }

    // Download binaries
    println!("Downloading binaries...");
    let binary_archive_path = PathBuf::from(BINARY_PATH_STR.to_owned() + &release + ".tar.xz");
    download_binaries(&release, &binary_archive_path).map_err(|err| {
        format!(
            "Could not download precompiled binaries for your kernel version.\nError: {}",
            err
        )
    })?;

    println!("Extracting binaries...");

    // Extract binaries
    let mut archive = Archive::new(XzDecoder::new(
        File::open(binary_archive_path).map_err(|_| "Could not open archive file")?,
    ));
    archive
        .unpack(BINARY_PATH_STR)
        .map_err(|err| format!("Could not unpack binary archive. Error: {}", err))?;

    // Prepare installation
    let binary_path = PathBuf::from(BINARY_PATH_STR.to_owned() + &release);

    // Install nexutil
    println!("Installing nexutil...");
    let nexutil_install_path = PathBuf::from("/usr/local/bin/nexutil");

    if nexutil_install_path.exists() {
        remove_file(&nexutil_install_path).map_err(|err| {
            format!(
                "nexutil is already installed and cannot be removed. Error: {}",
                err
            )
        })?;
    }

    let mut nexutil_path = binary_path.clone();
    nexutil_path.push("nexutil/nexutil");
    symlink(&nexutil_path, &nexutil_install_path)
        .map_err(|err| format!("Could not link nexutil. Error: {}", err))?;

    // Install makecsiparams
    println!("Installing makecsiparams...");

    let mcp_install_path = PathBuf::from("/usr/local/bin/mcp");
    let mcp_long_install_path = PathBuf::from("/usr/local/bin/makecsiparams");

    if mcp_install_path.exists() {
        remove_file(&mcp_install_path).map_err(|err| {
            format!(
                "mcp is already installed and cannot be removed. Error: {}",
                err
            )
        })?;
    }

    if mcp_long_install_path.exists() {
        remove_file(&mcp_long_install_path).map_err(|err| {
            format!(
                "makecsiparams is already installed and cannot be removed. Error: {}",
                err
            )
        })?;
    }

    let mut mcp_path = binary_path.clone();
    mcp_path.push("makecsiparams/makecsiparams");
    symlink(&mcp_path, &mcp_install_path)
        .map_err(|err| format!("Could not link mcp. Error: {}", err))?;
    symlink(&mcp_path, &mcp_long_install_path)
        .map_err(|err| format!("Could not link makecsiparams. Error: {}", err))?;

    // Unblock WiFi
    println!("Setting up WiFi...");
    Command::new("rfkill")
        .args(["unblock", "all"])
        .status()
        .map_err(|err| format!("Could not unblock WiFi. Error: {}", err))?;

    // Set WiFi country
    Command::new("raspi-config")
        .args(["nonint", "do_wifi_country", "US"])
        .status()
        .map_err(|err| format!("Could not set WiFi country. Error: {}", err))?;

    // Expand storage
    println!("Expanding storage...");
    Command::new("raspi-config")
        .args(["nonint", "do_expand_rootfs"])
        .status()
        .map_err(|err| format!("Could not expand storage. Error: {}", err))?;

    // Remember installation state
    File::create(NEXMON_INSTALLED_STR)
        .map_err(|err| format!("Could not save installation state. Error: {}", err))?;

    println!("Installation successful.");

    Ok(())
}

pub fn is_installed() -> bool {
    let is_installed_path = PathBuf::from(NEXMON_INSTALLED_STR);

    // Check if nexmon_csi has already been installed
    is_installed_path.exists()
}

fn download_binaries(release: &str, path: &PathBuf) -> Result<(), String> {
    // request URL
    let url = "https://github.com/nexmonster/nexmon_csi_bin/raw/main/base/".to_owned()
        + release
        + ".tar.xz";
    let mut response =
        reqwest::blocking::get(url).map_err(|err| format!("Error requesting URL: {}", err))?;

    // check if request returned an error code
    response
        .error_for_status_ref()
        .map_err(|err| format!("Error requesting URL: {}", err))?;

    let mut file = std::fs::File::create(path)
        .map_err(|err| format!("Could not create file to download archive: {}", err))?;
    response
        .copy_to(&mut file)
        .map_err(|err| format!("Error writing downloaded archive to file: {}", err))?;

    Ok(())
}
