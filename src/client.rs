use std::{io::Error, net::Ipv4Addr};

use crate::{arp::ArpMessage, interfaces::Interface};
use pnet::{
    datalink::{channel, Channel, DataLinkReceiver},
    packet::{
        arp::ArpPacket,
        ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
    },
    util::MacAddr,
};

pub struct ArpClient {
    rx_channel: Box<dyn DataLinkReceiver>,
    interface: Interface,
}

impl ArpClient {
    pub fn new() -> Self {
        ArpClient::new_with_iface(&Interface::new())
    }

    pub fn new_with_iface(interface: &Interface) -> Self {
        let rx = match channel(&interface.get_raw_interface(), Default::default()) {
            Ok(Channel::Ethernet(_, rx)) => rx,
            Ok(_) => panic!("Unknown channel type"),
            Err(err) => panic!("Error when opening channel: {}", err),
        };

        ArpClient {
            rx_channel: rx,
            interface: interface.clone(),
        }
    }

    pub fn ip_to_mac(&mut self, ip_addr: Ipv4Addr) -> Result<MacAddr, Error> {
        let message =
            ArpMessage::new_arp_request(self.interface.get_mac(), self.interface.get_ip(), ip_addr);

        match message.send(&self.interface) {
            Err(e) => return Err(e),
            _ => {}
        }

        loop {
            match self.next() {
                Some(arp_message) if arp_message.source_protocol_address == ip_addr => {
                    return Ok(arp_message.source_hardware_address)
                }
                _ => {}
            }
        }
    }

    pub fn mac_to_ip(&mut self, mac_addr: MacAddr) -> Result<Ipv4Addr, Error> {
        let message = ArpMessage::new_rarp_request(self.interface.get_mac(), mac_addr);

        match message.send(&self.interface) {
            Err(e) => return Err(e),
            _ => {}
        }

        loop {
            match self.next() {
                Some(arp_message) if arp_message.source_hardware_address == mac_addr => {
                    return Ok(arp_message.target_protocol_address)
                }
                _ => {}
            }
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
}
