# cspi for CSI Extraction on Raspberry Pi

This tool exists to simplify the extraction of Channel State Information (CSI) from the WiFi Chip of a Raspberry Pi using the [nexmon_csi](https://github.com/seemoo-lab/nexmon_csi) tool provided by the Secure Mobile Networking Lab (SEEMO) of the TU Darmstadt. It lets you install, apply and enable CSI collection using simple commands as well as easily restore the original firmware (to re-enable WiFi capability) and collect CSI data to a file for later analysis.

## Setting up for CSI collection

SEEMO provides nexmon CSI sources and the tools to compile them yourself. However, this process can be quite complicated, take a long time and require large downloads. To combat this, the nexmon user group [nexmonster](https://github.com/nexmonster) provides precompiled binaries for certain setups. This tool is currently only able to install these precompiled binaries. Compilation from source when precompiled binaries are not available may be added as a feature in later versions.

### Setting up your Raspberry Pi

As the WiFi driver patch is kernel version dependent, it is essential you install the exact required kernel version. The easiest way to do this is to install a release from the RaspiOS archives:

RaspiOS variant                         | Link
----------------------------------------|------------------------------------
Full (including various software)       | https://downloads.raspberrypi.org/raspios_full_armhf/images/raspios_full_armhf-2022-01-28/
Lite (barebones install)                | https://downloads.raspberrypi.org/raspios_lite_armhf/images/raspios_lite_armhf-2022-01-28/

**Note**: When going through the Raspberry Pi's first time setup prompt, be sure to **skip** the "Update Software" portion of the setup, as this would update the kernel and kernel headers, making the nexmon patch unusable. Never update the kernel even later on, so **do not** blindly run `apt upgrade` even if another tool tells you to before installation.

### Installing cspi
Download the latest binary of cspi from the Releases section, then run the following commands:

```bash
sudo apt update
sudo apt install libssl-dev tcpdump
sudo install <path to binary> /usr/bin/
```

If you want, you can also compile cspi from sources yourself. This is usually unnecessary as the binary available in Releases is always the latest version, compiled for the Raspberry Pi. If you do wish to compile it, make sure you have a rust toolchain installed, following the instructions [here](https://www.rust-lang.org/tools/install) if necessary, and clone this repository. Then, from the repository root, install the dependencies, build the binary and install it:
```bash
sudo apt update
sudo apt install libssl-dev tcpdump protobuf-compiler
cargo build --release  
sudo install target/release/cspi /usr/bin/
```

**Note:** cspi installs its patches and config files in `/home/pi/`. You must thus be using the standard `pi` user.

### Installing the precompiled nexmon binary

To install nexmon CSI, run the following command:  
```bash
sudo cspi install
```  

This step requires a working internet connection.

## Using Nexmon CSI
### Applying the nexmon patch
Once Nexmon is installed, you can apply the firmware patch. This only needs to be done once (unless you have restored the original firmware at some point, in which case you must do it again before being able to collect CSI).  
To apply the patch, run the following command:  
```bash
sudo cspi apply
```  

**Note:** If you are connected to the Raspberry Pi via SSH, make sure to do so via an Ethernet connection, as Nexmon CSI will break your WiFi connection.

### Protocol Buffers
By default, this tool outputs collected CSI as UDP packets encoded with Google's [Protocol Buffers](https://protobuf.dev/). Decoding it for your application is as easy as finding your chosen language's protobuf implementation, including it in your project, passing the packet to it to be decoded, and then receiving the decoded data parsed into the data structures of your chosen language - no custom parsing required!

To teach the format to your application, you must include the .proto file from the root of this repository in your project. Your language's protobuf implementation can then generate code from it that will do the decoding and provide the data for you.

### Collecting CSI
To enable CSI collection, run the following:  
```bash
sudo cspi start -c <channel> -b <bandwidth> -m <maclist>
```  
CSI in the default nexmon format will be available on UDP port 5500. The decoded CSI in the protobuf format will be available on port 4400. If you do not provide channel and/or bandwidth information, cspi will default to channel 36 and/or bandwidth 80. The maclist argument takes a comma separated list of mac addresses (in format `11:11:11:11:11:11,22:22:22:22:22:22,33:33:33:33:33:33`) which packets to evaluate CSI from should be sent from. If none are provided, CSI will be evaluated from all sources.

You can stop CSI collection like this:  
```bash
sudo cspi stop
```

If you wish to collect a certain number of packets into a pcap file for later analysis, you can run the following:  
```bash
sudo cspi collect -c <channel> -b <bandwidth> -m <maclist> -p <number of packets> -o <file name/path>
```  
If you do not provide packet number, it defaults to 1000. If you do not provide a file name/path, it defaults to output.pcap in the current working directory.
If you need your data encoded in the original nexmon format for compatibility with legacy tools, specify the `-n` flag. Otherwise, data will be encoded as a series of 32-bit message length and then protobuf message.

**Note:** If CSI collection does not return any packets even though you are sure there is traffic on the selected channel (and from filtered MAC addresses), it is possible the firmware has crashed. Run `sudo cspi restore` and then `sudo cspi apply`.

If the decoder that translates nexmon_csi data to the protobuf format stopped for any reason, you can restart it with
```bash
sudo cspi decode
```
This will do nothing if the decoder is already running.

### Disabling Nexmon CSI
If you wish to use the Pi's WiFi functionality again, you can restore the original WiFi firmware as follows:  
```bash
sudo cspi restore
```
After a few seconds, the Pi should be able to reconnect to regular WiFi networks and access the internet.

## Analyzing the CSI
We provide two example applications written in python to showcase the two modes of operation (reading live from the port or using collected .pcap files). To use either of the example applications, clone this repository (or download the respective folder).

The live visualizer reads from port 4400 live once per second and visualizes the packet of received CSI as two graphs (one for amplitude over subcarrier, one for phase over subcarrier). Run by entering the `live_visualizer` directory and running `python live_visualizer.py`.

The spectrogram visualizer takes a pcap file of packets encoded in the protobuf format (the default format). It visualizes amplitude of the complex CSI across the subcarriers over time as a spectrogram. Run by entering the `spectrogram_visualizer` directory and running `python spectrogram_visualizer.py <path-to-pcap>`.

To use the example applications, some additional packages need to be installed. On a Raspberry Pi running nexmon_csi (and thus using an old version of RaspiOS) is important to install protobuf from pip so it's up-to-date enough to parse the protobuf messages correctly, but numpy and matplotlib must be installed from the RaspiOS repositories as they otherwise will not run correctly on the Raspberry Pi:
```bash
pip install protobuf
sudo apt install python3-numpy python3-matplotlib
```

If you are for example parsing a CSI file on another device, you can install these packages from any up-to-date repository.

Additionally, SEEMO and nexmonster provide tools that take pcap files recorded with the original nexmon format (i.e. with the `-n` flag specified when using `cspi collect`) as their input:

SEEMO provides a [matlab tool](https://github.com/seemoo-lab/nexmon_csi/tree/master/utils/matlab) for reading the packet format and analyzing the collected CSI. See the accompanying [usage instructions](https://github.com/seemoo-lab/nexmon_csi/tree/master#analyzing-the-csi) in the nexmon_csi repo.

Nexmonster additionally provides a [python tool](https://github.com/nexmonster/nexmon_csi/tree/feature/python/utils/python) to read and visualize the packets. To use, copy your pcap file into the `pcapfiles/` subfolder and run `python csiexplorer.py` (or `python3 csiexplorer.py` if you haven't installed python-is-python3). Provide the file name (you do not need to enter the .pcap ending), press Enter and then enter the number of packet you want to see (0-indexed) or a range of packets that will be printed and visualized one after the other.