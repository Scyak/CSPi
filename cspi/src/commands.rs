pub mod apply;
pub mod collect;
pub mod install;
pub mod restore;
pub mod start;
pub mod stop;
pub mod running;
pub mod decode;

pub const BINARY_PATH_STR: &str = "/home/pi/.cspi/bins/";
pub const FIRMWARE_PATCHED_STR: &str = "/home/pi/.cspi/firmware_patched";
pub const NEXMON_RUNNING_STR: &str = "/home/pi/.cspi/nexmon-running";
pub const NEXMON_DECODER_PID_STR: &str = "/home/pi/.cspi/nexmon-decoder.pid";