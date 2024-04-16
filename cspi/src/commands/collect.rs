use std::net::UdpSocket;
use std::path::PathBuf;
use std::process::Command;
use std::fs::{remove_file, OpenOptions};
use std::io::Write;
use spinner::SpinnerBuilder;

use crate::commands::running::is_running;
use crate::commands::start::start;
use crate::commands::stop::stop;

pub fn collect(
    channel: &u32,
    bandwidth: &u32,
    maclist: &String,
    packets: &u32,
    output: PathBuf,
    nexmon: bool,
) -> Result<(), String> {
    println!("Collecting {} packets of CSI...", packets);

    // stop CSI collection if it is running (may be running with other parameters, so restart later in that case)
    if is_running() {
        stop()?;
    }

    let mut output_str: String = output.to_str().expect("Could not convert path to string").into();

    if output_str.ends_with(".pcap") && !nexmon {
        output_str = output_str + ".csi";
    } else if nexmon && !output_str.ends_with(".pcap") {
        output_str = output_str + ".pcap";
    }

    // start CSI collection
    start(channel, bandwidth, maclist)?;

    // dump requested number of packets to specified file
    if nexmon {
        Command::new("tcpdump")
            .args([
                "-i", "wlan0", "dst", "port", "5500", "-vv", "-w", &output_str, "-c",
            ])
            .arg(format!("{}", packets))
            .status()
            .map_err(|err| format!("Error running tcpdump: {}", err))?;
    } else {
        listen(&output_str, *packets);
    }

    // stop CSI collection
    stop()?;

    println!(
        "All done! Your collected CSI is available at {}",
        output_str
    );

    Ok(())
}

fn listen(output_file: &str, packet_num: u32) {
    let nexmon_socket = UdpSocket::bind("127.0.0.1:4400").unwrap();
    nexmon_socket.set_read_timeout(None).unwrap();

    let _ = remove_file(output_file);
    let mut file = OpenOptions::new().create_new(true).append(true).open(output_file).unwrap();

    let spinner = SpinnerBuilder::new("Collecting packets...".into()).start();

    for i in 1..=packet_num {
        let mut packet = [0; 8192];
        let packet_len: usize;
        match nexmon_socket.recv_from(&mut packet) {
            Err(_) => continue,
            Ok(len) => {packet_len = len.0},
        }

        file.write_all(&usize::to_le_bytes(packet_len)).unwrap_or_default();
        file.write_all(&packet[..packet_len]).unwrap_or_default();
        let _ = file.flush();

        if i % 10 == 0 {
            spinner.update(format!("Collected packets: {}", i));
        }
    }

    println!();
}
