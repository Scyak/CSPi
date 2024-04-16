use std::path::PathBuf;

use super::NEXMON_RUNNING_STR;

pub fn running() {
    match is_running() {
        true => println!("CSI collection is currently running."),
        false => println!("CSI collection is not running."),
    }
}

pub fn is_running() -> bool {
    let is_running_path = PathBuf::from(NEXMON_RUNNING_STR);

    // Check if nexmon_csi has already been installed
    is_running_path.exists()
}
