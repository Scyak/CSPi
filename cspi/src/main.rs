use clap::{Parser, Subcommand};
use commands::collect::collect;
use commands::decode::decode;
use commands::install::install;
use commands::restore::restore;
use commands::running::running;
use commands::start::start;
use commands::stop::stop;
use commands::{apply::apply, decode::launch_decoder};
use std::path::PathBuf;

mod commands;

pub mod csi {
    include!(concat!(env!("OUT_DIR"), "/csi.rs"));
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// downloads and installs precompiled nexmon binary
    Install {
        /// will install even if cspi already detects a compiled binary (use to update or in case of corrupted install)
        #[arg(short, long)]
        force: bool,
    },
    /// applies firmware patch (disabling wifi)
    Apply {},
    /// restores original firmware and re-enables wifi
    Restore {},
    /// collects CSI into pcap file according to specified parameters
    Collect {
        /// wifi channel to collect CSI on
        #[arg(short, long, default_value_t = 36)]
        channel: u32,
        /// bandwidth to use: 20, 40 or 80
        #[arg(short, long, default_value_t = 80)]
        bandwidth: u32,
        /// comma separated list of source mac addresses to evaluate packets from
        #[arg(short, long, default_value_t = String::from(""))]
        maclist: String,
        /// number of packets to collect
        #[arg(short, long, default_value_t = 1000)]
        packets: u32,
        /// path of output file [default: ./output.pcap]
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// changes output format to original nexmon format
        #[arg(short, long, action)]
        nexmon: bool,
    },
    /// starts CSI collection according to specified parameters (CSI will be available in nexmon format on UDP port 5500 and in protobuf format port 4400)
    Start {
        /// wifi channel to collect CSI on
        #[arg(short, long, default_value_t = 36)]
        channel: u32,
        /// bandwidth to use: 20, 40 or 80
        #[arg(short, long, default_value_t = 80)]
        bandwidth: u32,
        /// comma separated list of source mac addresses to evaluate packets from
        #[arg(short, long, default_value_t = String::from(""))]
        maclist: String,
    },
    /// stops CSI collection
    Stop {},
    /// tells you whether CSI collection is currently running
    Running {},
    /// starts the decoder and outputs protobuf messages on port 4400
    Decode {},
    /// test
    Testdecode {},
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { force } => {
            install(force).map_err(|err| format!("Installation unsuccessful. Error: {}", err))?
        }
        Commands::Apply {} => {
            apply().map_err(|err| format!("Could not apply firmware patch. Error: {}", err))?
        }
        Commands::Restore {} => restore()
            .map_err(|err| format!("Could not restore original firmware. Error: {}", err))?,
        Commands::Collect {
            channel,
            bandwidth,
            maclist,
            packets,
            output,
            nexmon,
        } => collect(
            channel,
            bandwidth,
            maclist,
            packets,
            output.clone().unwrap_or(PathBuf::from("output.pcap")),
            *nexmon,
        )
        .map_err(|err| format!("Could not collect the requested packets. Error: {}", err))?,
        Commands::Start {
            channel,
            bandwidth,
            maclist,
        } => start(channel, bandwidth, maclist)
            .map_err(|err| format!("Could not start CSI collection. Error: {}", err))?,
        Commands::Stop {} => {
            stop().map_err(|err| format!("Could not stop CSI collection. Error: {}", err))?
        }
        Commands::Running {} => running(),
        Commands::Decode {} => launch_decoder().map_err(|err| {
            format!(
                "Decoder not launched. The most likely cause is that it was already running. Error: {}",
                err
            )
        })?,
        Commands::Testdecode {} => decode(),
    }

    Ok(())
}
