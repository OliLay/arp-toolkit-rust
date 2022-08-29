use crate::arp;
use crate::interfaces::MacAddr;
use crate::{arp::ArpMessage, interfaces::Interface};
use pnet::{
    datalink::DataLinkReceiver,
    packet::{
        arp::ArpPacket,
        ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket},
    },
};
use std::convert::TryInto;
use std::time::Duration;
use std::{
    io::{Error, ErrorKind},
    net::Ipv4Addr,
    time::Instant,
};

/// Struct that encapsulates interaction with (R)ARP messages, such as sending and receiving.
pub struct ArpClient {
    rx_channel: Box<dyn DataLinkReceiver>,
    interface: Interface,
}

impl ArpClient {
    /// Create an ARP client on a guessed, "best-suited" interface.
    pub fn new() -> Result<Self, Error> {
        ArpClient::new_with_iface(&Interface::new())
    }

    /// Create an ARP client on the interface with the name `iface_name`.
    pub fn new_with_iface_name(iface_name: &str) -> Result<Self, Error> {
        let iface = Interface::new_by_name(iface_name);

        match iface {
            Some(iface) => ArpClient::new_with_iface(&iface),
            None => Err(Error::new(ErrorKind::NotFound, "No such interface.")),
        }
    }

    /// Create an ARP client on the `interface` given.
    pub fn new_with_iface(interface: &Interface) -> Result<Self, Error> {
        let result = interface.create_tx_rx_channels();

        match result {
            Ok((_, rx)) => Ok(ArpClient {
                rx_channel: rx,
                interface: interface.clone(),
            }),
            Err(err) => Err(err),
        }
    }

    /// Send an ARP `message` with the given `timeout`.
    /// Returns the next ARP message received. (must not necessarily be related to your message sent)
    #[maybe_async::maybe_async]
    pub async fn send_message(
        &mut self,
        timeout: Option<Duration>,
        message: ArpMessage,
    ) -> Result<ArpMessage, Error> {
        self.send_message_with_check(timeout, message, |arp_message| Some(arp_message))
            .await
    }

    /// Send an ARP `message` with the given `timeout`, and perform an arbitrary check `check_answer` on the answer.
    /// Using `check_answer`, you can check if the received tmessage is related to your previously sent message if needed.
    /// Returns the first ARP message received that satisfies `check_answer`.
    #[maybe_async::maybe_async]
    pub async fn send_message_with_check<T>(
        &mut self,
        timeout: Option<Duration>,
        message: ArpMessage,
        check_answer: impl Fn(ArpMessage) -> Option<T>,
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
            match self.receive_next().await {
                Some(arp_message) => match check_answer(arp_message) {
                    Some(result) => return Ok(result),
                    None => {}
                },
                _ => {}
            }
        }

        return Err(Error::new(ErrorKind::TimedOut, "Timeout"));
    }

    /// Resolves a given `ip_addr` to a MAC address.
    /// To achieve this, sends an ARP request with a `timeout`.
    #[maybe_async::maybe_async]
    pub async fn ip_to_mac(
        &mut self,
        ip_addr: Ipv4Addr,
        timeout: Option<Duration>,
    ) -> Result<MacAddr, Error> {
        let message = ArpMessage::new_arp_request(
            self.interface.get_mac().into(),
            self.interface.get_ip().unwrap(),
            ip_addr,
        );

        self.send_message_with_check(timeout, message, |arp_message| {
            return if arp_message.source_protocol_address == ip_addr
                && arp_message.operation == arp::Operation::ArpResponse
            {
                Some(arp_message.source_hardware_address.into())
            } else {
                None
            };
        })
        .await
    }

    /// Resolves a given `mac_addr` to an IPv4 address.
    /// To achieve this, sends an RARP request with a `timeout`.
    #[maybe_async::maybe_async]
    pub async fn mac_to_ip(
        &mut self,
        mac_addr: MacAddr,
        timeout: Option<Duration>,
    ) -> Result<Ipv4Addr, Error> {
        let message =
            ArpMessage::new_rarp_request(self.interface.get_mac().into(), mac_addr.into());

        self.send_message_with_check(timeout, message, |arp_message| {
            let source_mac: MacAddr = arp_message.source_hardware_address.into();

            if source_mac == mac_addr && arp_message.operation == arp::Operation::RarpResponse {
                Some(arp_message.target_protocol_address)
            } else {
                None
            }
        })
        .await
    }

    /// Sends `arp_message` on the interface belonging to this client.
    #[maybe_async::maybe_async]
    pub async fn send(&self, arp_message: &ArpMessage) -> Result<(), Error> {
        arp_message.send(&self.interface)
    }

    /// Returns when the next Ethernet frame has been received. If this frame contains an ARP message,
    /// returns this message, else returns None.
    #[maybe_async::maybe_async]
    pub async fn receive_next(&mut self) -> Option<ArpMessage> {
        let rx_ethernet_packet = self.next_ethernet_frame().await;

        match rx_ethernet_packet {
            Some((packet, bytes))
                if packet.get_ethertype() == EtherTypes::Arp
                    || packet.get_ethertype() == EtherTypes::Rarp =>
            {
                let arp_packet =
                    ArpPacket::new(&bytes[MutableEthernetPacket::minimum_packet_size()..]).unwrap();

                match arp_packet.try_into() {
                    Ok(arp_packet) => return Some(arp_packet),
                    Err(_) => return None,
                }
            }
            _ => return None,
        }
    }

    #[maybe_async::maybe_async]
    async fn next_ethernet_frame(&mut self) -> Option<(EthernetPacket<'_>, &[u8])> {
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
