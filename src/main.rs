mod arp;
mod interfaces;

use arp::ArpMessage;
use pnet::packet::arp::ArpHardwareTypes;
use pnet::packet::arp::{ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::{
    datalink::{channel, Channel, MacAddr},
    packet::{arp::ArpOperation, MutablePacket, Packet},
};
use std::net::Ipv4Addr;

fn main() {
    let interfaces = interfaces::get_all_interfaces();
    let iface = interfaces::get_interface_by_guess(&interfaces).unwrap();

    let arp_message = ArpMessage::new_rarp_request(
        iface.mac.unwrap(),
        MacAddr::new(0xdc, 0xa6, 0x32, 0x27, 0x5b, 0xd8),
    );

    arp_message.send(iface).unwrap();
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
