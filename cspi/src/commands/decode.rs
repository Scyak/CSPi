use crate::commands::NEXMON_DECODER_PID_STR;
use crate::csi::{Csi, NexmonData};
use daemonize::{Daemonize, Outcome::Child};
use prost::Message;
use std::fs;
use std::net::UdpSocket;
use std::path::Path;
use std::process::Command;

/// launches decoder if it's not already running
pub fn launch_decoder() -> Result<(), String> {
    let pgrep_output = String::from_utf8(
        Command::new("pgrep")
            .args(["cspi"])
            .output()
            .map_err(|err| format!("Could not obtain information from pgrep. Error: {}", err))?
            .stdout,
    )
    .map_err(|_| "Could not parse string returned by pgrep cspi")?;

    let daemon_pid = match fs::read_to_string(NEXMON_DECODER_PID_STR) {
        Ok(path) => path,
        Err(_) => {
            if !Path::new(NEXMON_DECODER_PID_STR).is_file() {
                String::from("NOT FOUND")
            } else {
                String::from("")
            }
        }
    };

    if !pgrep_output.contains(&daemon_pid) {
        match Daemonize::new().pid_file(NEXMON_DECODER_PID_STR).execute() {
            Child(_) => {
                decode();
                return Err(String::from("Decoding stopped!"));
            }
            _ => {
                println!("Launched decoder.")
            }
        }
    } else {
        println!("Decoder already running.")
    }

    Ok(())
}

pub fn decode() {
    // set up read and write streams
    let nexmon_socket = UdpSocket::bind("255.255.255.255:5500").unwrap();
    nexmon_socket.set_read_timeout(None).unwrap();
    let output_socket = UdpSocket::bind("127.0.0.1:4401").unwrap();

    loop {
        // read from 5500
        let mut message_buffer = [0; 4096];
        let received_bytes;
        match nexmon_socket.recv_from(&mut message_buffer) {
            Err(_) => continue,
            Ok(ok) => received_bytes = ok.0,
        }

        // decode CSI
        let mut nexmon_data = NexmonData {
            csi: vec![],
            rssi: i8::from_le_bytes([message_buffer[2]]) as i32,
            fctl: u8::from_le_bytes([message_buffer[3]]) as u32,
            source_mac: u64::from_be_bytes([
                0,
                0,
                message_buffer[4],
                message_buffer[5],
                message_buffer[6],
                message_buffer[7],
                message_buffer[8],
                message_buffer[9],
            ]),
            seq_num: u16::from_le_bytes([message_buffer[10], message_buffer[11]]) as u32
        };

        let mut csi = vec![];
        for csi_subcarrier in message_buffer[18..received_bytes].chunks(4) {
            let real = i16::from_le_bytes([csi_subcarrier[0], csi_subcarrier[1]]) as i32;
            let imaginary = i16::from_le_bytes([csi_subcarrier[2], csi_subcarrier[3]]) as i32;
            let csi_element = Csi { real, imaginary };
            csi.push(csi_element);
        }

        // halves of csi need to be swapped
        let subcarrier_num = csi.len();
        nexmon_data.csi = csi[(subcarrier_num/2)..].iter().chain(csi[..subcarrier_num/2].iter()).cloned().collect();

        // encode as protobuf message
        let encoded_vec = nexmon_data.encode_to_vec();

        // write to 4400
        output_socket
            .send_to(&encoded_vec[..], "127.0.0.1:4400")
            .unwrap_or_default();
    }
}
