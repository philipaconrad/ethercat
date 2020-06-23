#!/bin/bash
set -eu

# A simple end-to-end test of EtherCat.

# A receiver instance is started up in the background, and then we a file is
# sent over the wire.
# The test result is determined by comparing the diff between checksums of
# the files. If they're different, we mangled something along the way.

ETHERCAT=${BASH_SOURCE%/*}/../target/debug/ethercat

# Start receiving ethercat instance.
sudo $ETHERCAT -l lo 00:00:00:00:00:00 > ${BASH_SOURCE%/*}/received.txt &

# Send ethercat's source code over as test content.
cat ${BASH_SOURCE%/*}/../src/main.rs | sudo $ETHERCAT lo 00:00:00:00:00:00 > /dev/null

sleep 0.25

md5sum ${BASH_SOURCE%/*}/../src/main.rs | cut -f1 -d' ' > ${BASH_SOURCE%/*}/original-cksum.txt
md5sum ${BASH_SOURCE%/*}/received.txt | cut -f1 -d' ' > ${BASH_SOURCE%/*}/received-cksum.txt

diff ${BASH_SOURCE%/*}/original-cksum.txt ${BASH_SOURCE%/*}/received-cksum.txt
