EtherCat
--------

EtherCat is a networking utility in the style of `netcat` for reading and writing to raw ethernet sockets on Linux.

# Build

    cargo build

# Run

    ethercat <your_ethernet_interface> <dest_mac_address>

Note: You may have to add `sudo` to this command in order to get permission to create raw ethernet sockets. Wireshark and other tools work around this restriction by creating a packet-capturing user group, but this whole group has root-level permissions, so this approach can be dangerous on production machines.

# Usage example

    cat README.md | sudo ethercat enp0s31f6 00:00:00:00:00:00

On my development machine, I have an ethernet port under the name `enp0s31f6`. If you have a live ethernet port (and `sudo` permissions!) you can see packets appear in Wireshark or other packet monitoring tools.

# License

See `LICENSE.txt` for the full BSD-3 license text.
