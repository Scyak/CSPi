import proto.csi_pb2 as csi_pb2
import matplotlib.pyplot as plt
import numpy as np
import time
import socket

null_subcarriers = {
    64: [0, 1, 2, 3, 32, 61, 62, 63],
    128: [0, 1, 2, 3, 4, 5, 63, 64, 65, 123, 124, 125, 126, 127],
    256: [0, 1, 2, 3, 4, 5, 127, 128, 129, 130, 131, 251, 252, 253, 254, 255]
}

if __name__ == "__main__":
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind(('127.0.0.1', 4400))

    fig, (amp, phase) = plt.subplots(2, 1)
    fig.suptitle('CSI')

    amp.set_ylabel('Amplitude')

    phase.set_xlabel('Phase')
    phase.set_ylabel('Subcarrier')

    plt.ion()

    while(1):
        data, _ = sock.recvfrom(8192)

        print("New packet:")

        # protobuf deserialization
        nexmon_data = csi_pb2.NexmonData()
        try:
            nexmon_data.ParseFromString(data)
        except:
            print("Failed to parse data!")
            continue

        print("RSSI: " + str(nexmon_data.rssi))
        print("FCTL: " + str(nexmon_data.fctl))

        mac_addr = ':'.join(['{}{}'.format(a, b)
                     for a, b
                     in zip(*[iter('{:012x}'.format(nexmon_data.source_mac))]*2)])
        print("Source MAC: " + mac_addr)
        print("Sequence number: " + str(nexmon_data.seq_num))
        print()

        complex_csi = []
        for csi_element in nexmon_data.csi:
            complex_csi.append((csi_element.real + 1.j * csi_element.imaginary))

        # remove null subcarriers, these often have absurly high amplitudes and are not useful data
        for null_carrier in null_subcarriers[len(complex_csi)]:
            complex_csi[null_carrier] = 0

        amp.clear()
        phase.clear()

        amp.set_ylabel('Amplitude')

        phase.set_xlabel('Phase')
        phase.set_ylabel('Subcarrier')

        amp.plot(np.abs(complex_csi))
        phase.plot(np.angle(complex_csi, deg=True))
        fig.canvas.draw()
        fig.canvas.flush_events()

        plt.pause(1)
