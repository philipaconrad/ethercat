#!/bin/bash
set -eu

# A simple end-to-end test of EtherCat.

ETHERCAT=${BASH_SOURCE%/*}/../target/debug/ethercat

# Find the default ethernet interface.
# Note: this command can fail if the ethernet interface name is over 8
#   characters long, because the default ifconfig on Ubuntu truncates names.
ETHER_INTERFACE=$(ifconfig -a | grep -Eo "^en[a-zA-Z0-9]*")
echo "Test will use interface: ${ETHER_INTERFACE}"

# The file is not truncated for some reason, so we have to manually remove it,
# or else it will grow if the test is run locally more than once.
rm -f ${BASH_SOURCE%/*}/received.txt

# Start the receiving ethercat instance.
sudo $ETHERCAT -l $ETHER_INTERFACE 00:00:00:00:00:00 > ${BASH_SOURCE%/*}/received.txt &

# Give reciever time to start up and bind socket.
sleep 0.5s

# Send ethercat's source code over to it as test material.
cat ${BASH_SOURCE%/*}/../src/main.rs | sudo $ETHERCAT -O 500 $ETHER_INTERFACE 00:00:00:00:00:00 > /dev/null

# Small delay to give receiver time to receive all packets.
sleep 0.5s

# Ensure background job dies and flushes writes to disk.
kill %%

# Generate MD5 hashes for the two files.
md5sum ${BASH_SOURCE%/*}/../src/main.rs | cut -f1 -d' ' > ${BASH_SOURCE%/*}/original-cksum.txt
md5sum ${BASH_SOURCE%/*}/received.txt | cut -f1 -d' ' > ${BASH_SOURCE%/*}/received-cksum.txt

# Compare the two hashes. Matching hashes mean nothing got mangled in-transit.
ls -athor ${BASH_SOURCE%/*}
diff ${BASH_SOURCE%/*}/original-cksum.txt ${BASH_SOURCE%/*}/received-cksum.txt
