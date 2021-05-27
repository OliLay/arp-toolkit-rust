use std::collections::btree_set::Intersection;

use crate::{arp::ArpMessage, interfaces::Interface};
use pnet::{
    datalink::{channel, Channel, DataLinkReceiver, NetworkInterface},
    packet::{
        arp::ArpPacket,
        ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
    },
};

pub struct ArpClient {
    rx_channel: Box<dyn DataLinkReceiver>,
}

impl ArpClient {
    pub fn new(interface: &Interface) -> Self {
        let rx = match channel(&interface.get_raw_interface(), Default::default()) {
            Ok(Channel::Ethernet(_, rx)) => rx,
            Ok(_) => panic!("Unknown channel type"),
            Err(err) => panic!("Error when opening channel: {}", err),
        };

        ArpClient { rx_channel: rx }
    }

    fn next_ethernet_frame(&mut self) -> Option<(EthernetPacket, &[u8])> {
        let rx_bytes = match self.rx_channel.next() {
            Ok(rx_bytes) => rx_bytes,
            Err(_) => return None,
        };

        match EthernetPacket::new(&rx_bytes) {
            Some(frame) => Some((frame, rx_bytes)),
            None => None,
        }
    }

    pub fn next(&mut self) -> Option<ArpMessage> {
        loop {
            let rx_ethernet_packet = self.next_ethernet_frame();

            match rx_ethernet_packet {
                Some((packet, bytes))
                    if packet.get_ethertype() == EtherTypes::Arp
                        || packet.get_ethertype() == EtherTypes::Rarp =>
                {
                    let arp_packet =
                        ArpPacket::new(&bytes[MutableEthernetPacket::minimum_packet_size()..])
                            .unwrap();

                    return Some(arp_packet.into());
                }
                _ => return None,
            }
        }
    }
}
