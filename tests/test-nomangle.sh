#!/bin/bash

# A simple end-to-end test of EtherCat.

# A receiver instance is started up in the background, and then we a file is
# sent over the wire.
# The test result is determined by comparing the diff between checksums of
# the files. If they're different, we mangled something along the way.

# Start receiving ethercat instance.
sudo ethercat lo 00:00:00:00:00:00 > received.txt &

# Send ethercat's source code over as test content.
cat ${BASH_SOURCE%/*}/../src/main.rs | sudo ethercat lo 00:00:00:00:00:00

cksum ${BASH_SOURCE%/*}/../src/main.rs > original-cksum.txt
cksum received.txt > received-cksum.txt

diff original-cksum.txt received-cksum.txt
