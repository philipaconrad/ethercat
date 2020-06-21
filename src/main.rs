// EtherCat top-level code.
// Copyright (c) 2020, Philip Conrad. All rights reserved.
// Released under the terms of the BSD-3 License.
// See LICENSE for details.

extern crate pnet;
use pnet::datalink::{self, NetworkInterface, DataLinkSender};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::{Packet};
use pnet::packet::ethernet::{EtherType, EthernetPacket, MutableEthernetPacket};
use pnet::util::{MacAddr};

//use itertools::Itertools;

use std::io::{BufReader, BufRead};
use std::thread;


const HELP_TEXT: &str = "EtherCat 1.0
Copyright (c) Philip Conrad <conradp@chariot-chaser.net>, 2020.
All rights reserved. Released under the BSD-3 license.

Usage:
  ethercat [OPTIONS] SOURCE_IF DEST_MAC

Arguments:
 SOURCE_IF  Host interface to send/receive packets from.
 DEST_MAC   Destination MAC to send/receive packets to/from.

Options:
 -I     Size for receive buffer. [Default: 4096]
 -O     Size for send buffer. [Default: 1500]
 -i     File to read input from. Send buffer automatically flushes
        on EOF, unless the --no-flush-eof option is provided.

Future:
 - Option to spoof sender MAC?
 - Option to spoof packet checksums?
 - Option(s) to allow VLAN tagging, and other tag stuff?
";

struct Args {
    help: bool,
    version: bool,
    number: u32,
    //in_files: Vec<String>,
    opt_number: Option<u32>,
    recv_mtu: u32,
    send_mtu: u32,
    free: Vec<String>,
}

fn parse_int(s: &str) -> Result<u32, String> {
    s.parse().map_err(|_| "not a number".to_string())
}

/*fn parse_filenames(s: &str) -> Result<Vec<String>, String> {
    let strs = s.parse().split(",").collect(Vec<&str>)
}*/

// This is a mess, but it abstracts over sending a packet with libpnet.
fn packet_send(tx: &mut Box<dyn DataLinkSender + 'static>,
               source: MacAddr,
               dest: MacAddr,
               ether_type: u16,
               payload: Vec<u8>) -> Result<(), std::io::Error> {
    let ether_struct = pnet::packet::ethernet::Ethernet {
        source: source,
        destination: dest,
        payload: payload,
        ethertype: EtherType::new(ether_type),
    };

    // Lots of construction shenanigans required, but it works?
    // Cite: https://github.com/libpnet/libpnet/pull/79
    let mut packet_buf = vec![0; EthernetPacket::packet_size(&ether_struct)];
    let mut packet = MutableEthernetPacket::new(&mut packet_buf[..]).unwrap();
    packet.populate(&ether_struct);

    tx.send_to(packet.packet(), None).unwrap()
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = pico_args::Arguments::from_env();
    // Arguments can be parsed in any order.
    let args = Args {
        // You can use a slice for multiple commands
        help: args.contains(["-h", "--help"]),
        // or just a string for a single one.
        version: args.contains("-V"),
        // Parses an optional value that implements `FromStr`.
        number: args.opt_value_from_str("--number")?.unwrap_or(5),
        // Parses an optional value that implements `FromStr`.
        opt_number: args.opt_value_from_str("--opt-number")?,
        //in_files: args.opt_value_from_str("-i", parse_filenames)?.unwrap_or(),
        
        // Parses an optional value using a specified function.
        recv_mtu: args.opt_value_from_str(["-I", "--recv-mtu"])?.unwrap_or(4096),
        send_mtu: args.opt_value_from_str(["-O", "--send-mtu"])?.unwrap_or(1500),
        //send_mtu: args.opt_value_from_fn(["-O", "--send-mtu"], parse_int)?.unwrap_or(1500),
        // Will return all free arguments or an error if any flags are left.
        free: args.free()?,
    };

    for arg in &args.free { eprintln!("Arg: {}", arg) }

    // Print help message and bail.
    // It should be okay to use process::exit() here, as no file descriptors
    // or other system resources are being used yet.
    if args.help {
        eprintln!("{}", HELP_TEXT);
        std::process::exit(0)
    }

    // Ensure we have arguments available for our source/dest.
    if args.free.len() < 2 {
        println!("{}", "Error: Need both SOURCE_IF and DEST_MAC arguments.");
        std::process::exit(1)
    }

    let source_if = String::from(args.free.get(0).unwrap());
    let dest_mac: MacAddr = (args.free.get(1).unwrap()).parse::<MacAddr>()?;

    // Filter network interfaces to find our link.
    let interface_names_match =
        |iface: &NetworkInterface| iface.name == source_if;

    // Find the network interface with the provided name
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter()
                              .filter(interface_names_match)
                              .next()
                              .unwrap();

    // Bail if we can't find a MAC address for the network interface.
    let source_mac = match interface.mac {
        Some(mac) => mac,
        None => panic!("No MAC address available for interface '{}'", interface.name),
    };

    // Create a new channel, dealing with layer 2 packets.
    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };


    // Sender thread.
    // TODO: Replace `1500` constants with the send_mtu variable.
    let sender = thread::Builder::new().name("stdin -> send".to_string()).spawn(move || {
        let mut s = BufReader::with_capacity(8 * 1024, std::io::stdin());
        let mut out_buffer = Vec::with_capacity(1500);

        // Continuously receive input from stdin, until EOF is hit.
        // Whenever we have full MTU's-worth of data, we flush it.
        loop {
            let out_length = out_buffer.len();
            // Flush all full-size packets that we can.
            if out_length >= 1500 {
                let res = packet_send(&mut tx, source_mac, dest_mac, 0x0806, out_buffer[0..1500].to_vec());
                if let Err(err) = res {
                    eprintln!("Packet Send Error: {}", err);
                }
                out_buffer = out_buffer[1500..].to_vec();
                continue;
            };
            // Read next batch from BufReader.
            let in_buffer = s.fill_buf().unwrap();
            let in_length = in_buffer.len();
            if in_length > 0 {
                for i in 0..in_length { out_buffer.push(in_buffer[i]) }
            }
            // If we've cleared out all the full-size packets, check to see if
            // we have exhausted the BufReader.
            if out_length <= 1500 {
                if in_length == 0 { break; }
                // Hack to ensure correct stopping for early EOFs.
                // The hack is required because the underlying reader is line-
                // oriented, and will stop early in some cases.
                if in_length < 8 * 1024 && in_buffer.last().unwrap() != &b'\n' { break; }
            }

            // Advance the BufReader by however many bytes we read into its
            // internal buffer on the last fill_buf() call.
            s.consume(in_length);
        }
        // Send last (usually small) packet.
        if out_buffer.len() > 0 {
            let res = packet_send(&mut tx, source_mac, dest_mac, 0x0806, out_buffer[0..].to_vec());
            if let Err(err) = res {
                eprintln!("Packet Send Error: {}", err);
            }
            //println!("Sending buffer!: {}", std::str::from_utf8(&out_buffer).unwrap());
        };
    }).unwrap();

    // Receiver thread.
    /*thread::Builder::new().name("recv -> stdout".to_string()).spawn(move || {
        let out_writer = BufferedWriter::new(io::stdio::stdout);
        loop {
            match rx.next() {
                Ok(packet) => {
                    let packet = EthernetPacket::new(packet).unwrap();
                    // Constructs a single packet, the same length as the the one received,
                    // using the provided closure. This allows the packet to be constructed
                    // directly in the write buffer, without copying. If copying is not a
                    // problem, you could also use send_to.
                    //
                    // The packet is sent once the closure has finished executing.
                    /*tx.build_and_send(1, packet.packet().len(),
                        &mut |mut new_packet| {
                            let mut new_packet = MutableEthernetPacket::new(new_packet).unwrap();

                            // Create a clone of the original packet.
                            new_packet.clone_from(&packet);

                            // Switch the source and destination.
                            new_packet.set_source(packet.get_destination());
                            new_packet.set_destination(packet.get_source());
                    });*/

                    out_writer.write(packet.payload());
                    out_writer.flush();
                },
                Err(e) => {
                    // If an error occurs, we can handle it here.
                    panic!("An error occurred while reading: {}", e);
                }
            }
        }
    });*/

    sender.join();

    Ok(())
}
