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

    let arp_message = ArpMessage::new_arp_request(
        interface.get_mac(),
        Ipv4Addr::new(192, 168, 178, 20),
        interface.get_ip(), //MacAddr::new(0x8c, 0x85, 0x90, 0x06, 0x68, 0xc9),
    );

    arp_message.send(&interface).unwrap();

    let mut client = ArpClient::new(&interface);

    loop {
        match client.next() {
            Some(arp_message) => {
                println!("le answer is {}", arp_message.source_hardware_address);
                return;
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
