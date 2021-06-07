use crate::interfaces::MacAddr;
use crate::{arp::ArpMessage, interfaces::Interface};
use pnet::{
    datalink::{channel, Channel, DataLinkReceiver},
    packet::{
        arp::ArpPacket,
        ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
    },
};
use std::time::Duration;
use std::{
    io::{Error, ErrorKind},
    net::Ipv4Addr,
    time::Instant,
};

pub struct ArpClient {
    rx_channel: Box<dyn DataLinkReceiver>,
    interface: Interface,
}

impl ArpClient {
    pub fn new() -> Self {
        ArpClient::new_with_iface(&Interface::new())
    }

    pub fn new_with_iface_name(iface_name: &str) -> Option<Self> {
        let iface = Interface::new_by_name(iface_name);

        match iface {
            Some(iface) => Some(ArpClient::new_with_iface(&iface)),
            None => None,
        }
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

    pub fn send_request(
        &mut self,
        timeout: Option<Duration>,
        message: ArpMessage,
    ) -> Result<ArpMessage, Error> {
        self.send_request_with_check(timeout, message, &|arp_message| Some(arp_message))
    }

    pub fn send_request_with_check<T>(
        &mut self,
        timeout: Option<Duration>,
        message: ArpMessage,
        check_answer: &dyn Fn(ArpMessage) -> Option<T>,
    ) -> Result<T, Error> {
        let unpacked_timeout = match timeout {
            Some(t) => t,
            // use Duration::MAX after integrated into Rust stable
            None => Duration::from_secs(u64::MAX),
        };

        match message.send(&self.interface) {
            Err(e) => return Err(e),
            _ => {}
        }

        let start_time = Instant::now();
        while Instant::now() - start_time < unpacked_timeout {
            match self.receive_next() {
                Some(arp_message) => match check_answer(arp_message) {
                    Some(result) => return Ok(result),
                    None => {}
                },
                _ => {}
            }
        }

        return Err(Error::new(ErrorKind::TimedOut, "Timeout"));
    }

    pub fn ip_to_mac(
        &mut self,
        ip_addr: Ipv4Addr,
        timeout: Option<Duration>,
    ) -> Result<MacAddr, Error> {
        let message = ArpMessage::new_arp_request(
            self.interface.get_mac().into(),
            self.interface.get_ip(),
            ip_addr,
        );

        self.send_request_with_check(timeout, message, &|arp_message| {
            return if arp_message.source_protocol_address == ip_addr {
                Some(arp_message.source_hardware_address.into())
            } else {
                None
            };
        })
    }

    pub fn mac_to_ip(
        &mut self,
        mac_addr: MacAddr,
        timeout: Option<Duration>,
    ) -> Result<Ipv4Addr, Error> {
        let message =
            ArpMessage::new_rarp_request(self.interface.get_mac().into(), mac_addr.into());

        self.send_request_with_check(timeout, message, &|arp_message| {
            let source_mac: MacAddr = arp_message.source_hardware_address.into();
            if source_mac == mac_addr {
                Some(arp_message.target_protocol_address)
            } else {
                None
            }
        })
    }

    pub fn send(&self, arp_message: &ArpMessage) -> Result<(), Error> {
        arp_message.send(&self.interface)
    }

    pub fn receive_next(&mut self) -> Option<ArpMessage> {
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
