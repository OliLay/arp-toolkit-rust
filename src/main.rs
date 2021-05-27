mod arp;
mod client;
mod interfaces;

use arp::ArpMessage;
use interfaces::Interface;
use client::ArpClient;
use pnet::packet::arp::ArpHardwareTypes;
use pnet::packet::arp::{ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::{
    datalink::{channel, Channel, MacAddr},
    packet::{arp::ArpOperation, MutablePacket, Packet},
};
use std::net::{IpAddr, Ipv4Addr};

fn main() {
    let interface = Interface::new();

    let searched_mac = MacAddr::new(0xdc, 0xa6, 0x32, 0x27, 0x5b, 0xd8);

    let arp_message = ArpMessage::new_rarp_request(
        interface.get_mac(),
        searched_mac,
    );

    arp_message.send(&interface).unwrap();

    let mut client = ArpClient::new(&interface);

    loop {
        match client.next() {
            Some(arp_message) if arp_message.source_hardware_address == searched_mac => {
                println!("le answer is {}", arp_message.source_protocol_address);
                return;
            }
            Some(_) => {
                println!("Received ARP message, but not an answer to before.")
            }
            None => {}
        }
    }

    /*
    while true {
        let rx_buf = rx.next().unwrap();
        let received_eth = EthernetPacket::new(&rx_buf);

        match received_eth {
            Some(packet) if packet.get_ethertype() == EtherTypes::Rarp => {
                let arp = ArpPacket::new(&rx_buf[MutableEthernetPacket::minimum_packet_size()..])
                    .unwrap();

                println!("Received reply {}", arp.get_sender_proto_addr());
                return;
            }
            Some(_) => {}
            None => {}
        }
    }
    */
}
