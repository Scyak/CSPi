import argparse
import os
import proto.csi_pb2 as csi_pb2
import matplotlib.pyplot as plt
import numpy as np

if __name__ == "__main__":
    parser = argparse.ArgumentParser(
                    prog='colormap_visualizer',
                    description='Creates spectrogram from passed CSI pcap file')
    
    parser.add_argument('csi_path', help="Path to the file of protobuf encoded CSI packets")
    args = parser.parse_args()

    with open(args.csi_path, 'rb') as pcap_file:
        pcap_data = pcap_file.read()
    pcap_file_length = os.path.getsize(args.csi_path)
    
    csi_list = []
    position = 0
    while position < pcap_file_length:
        payload_length = int.from_bytes(pcap_data[position : (position + 4)], byteorder='little', signed=False)
        position = position + 4

        # protobuf deserialization
        nexmon_data = csi_pb2.NexmonData()
        try:
            nexmon_data.ParseFromString(pcap_data[position : (position + payload_length)])
        except:
            position = position + payload_length
            print("Failed")
            continue

        position = position + payload_length
        csi_list.append(nexmon_data.csi)

    data = []

    for csi in csi_list:
        element_list = []
        for idx, csi_element in enumerate(csi):
            if idx > 2:
                element_list.append((csi_element.real + 1.j * csi_element.imaginary))
        data.append(element_list)

    fig, ax = plt.subplots()
    pcm = ax.pcolormesh(np.abs(data))
    plt.xlabel('Subcarrier')
    plt.ylabel('Packet number')
    plt.savefig('output.png')
