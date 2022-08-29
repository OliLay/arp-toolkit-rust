use crate::interfaces::{Interface, MacAddr};
use std::{
    convert::TryFrom,
    io::{Error, ErrorKind},
    net::Ipv4Addr,
    u16,
};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use pnet::packet::{
    arp::{ArpHardwareTypes, ArpOperation, ArpPacket, MutableArpPacket},
    ethernet::{
        EtherType,
        EtherTypes::{self},
        MutableEthernetPacket,
    },
    MutablePacket, Packet,
};

pub struct ArpMessage {
    pub source_hardware_address: MacAddr,
    pub source_protocol_address: Ipv4Addr,

    pub target_hardware_address: MacAddr,
    pub target_protocol_address: Ipv4Addr,

    pub ethertype: EtherType,
    pub operation: Operation,
}

#[derive(Copy, Clone, FromPrimitive, PartialEq)]
pub enum Operation {
    ArpRequest = 0x1,
    ArpResponse = 0x2,
    RarpRequest = 0x3,
    RarpResponse = 0x4,
}

impl ArpMessage {
    /// Constructs a new ARP message with arbitrary field contents.
    pub fn new(
        ethertype: EtherType,
        source_hardware_address: MacAddr,
        source_protocol_address: Ipv4Addr,
        target_hardware_address: MacAddr,
        target_protocol_address: Ipv4Addr,
        operation: Operation,
    ) -> Self {
        ArpMessage {
            source_hardware_address: source_hardware_address,
            source_protocol_address: source_protocol_address,
            target_hardware_address: target_hardware_address,
            target_protocol_address: target_protocol_address,
            ethertype: ethertype,
            operation: operation,
        }
    }

    /// Constructs a new ARP request message.
    pub fn new_arp_request(
        source_hardware_address: MacAddr,
        source_protocol_address: Ipv4Addr,
        target_protocol_address: Ipv4Addr,
    ) -> Self {
        Self::new(
            EtherTypes::Arp,
            source_hardware_address,
            source_protocol_address,
            MacAddr(0, 0, 0, 0, 0, 0),
            target_protocol_address,
            Operation::ArpRequest,
        )
    }

    /// Constructs a new ARP response message.
    pub fn new_arp_response(
        source_hardware_address: MacAddr,
        source_protocol_address: Ipv4Addr,
        target_hardware_address: MacAddr,
        target_protocol_address: Ipv4Addr,
    ) -> Self {
        Self::new(
            EtherTypes::Arp,
            source_hardware_address,
            source_protocol_address,
            target_hardware_address,
            target_protocol_address,
            Operation::ArpResponse,
        )
    }

    /// Constructs a new RARP request message.
    pub fn new_rarp_request(
        source_hardware_address: MacAddr,
        target_hardware_address: MacAddr,
    ) -> Self {
        Self::new(
            EtherTypes::Rarp,
            source_hardware_address,
            Ipv4Addr::new(0, 0, 0, 0),
            target_hardware_address,
            Ipv4Addr::new(0, 0, 0, 0),
            Operation::RarpRequest,
        )
    }

    /// Constructs a new RARP response message.
    pub fn new_rarp_response(
        source_hardware_address: MacAddr,
        source_protocol_address: Ipv4Addr,
        target_hardware_address: MacAddr,
        target_protocol_address: Ipv4Addr,
    ) -> Self {
        Self::new(
            EtherTypes::Rarp,
            source_hardware_address,
            source_protocol_address,
            target_hardware_address,
            target_protocol_address,
            Operation::RarpResponse,
        )
    }

    /// Sends the message on the given interface.
    /// # Errors
    /// Returns an error when sending fails.
    pub fn send(&self, interface: &Interface) -> Result<(), Error> {
        let mut tx = match interface.create_tx_rx_channels() {
            Ok((tx, _)) => tx,
            Err(err) => return Err(err),
        };

        let mut eth_buf = vec![0; 42];
        let mut eth_packet = MutableEthernetPacket::new(&mut eth_buf).unwrap();

        eth_packet.set_destination(MacAddr::new(0xff, 0xff, 0xff, 0xff, 0xff, 0xff).into());
        eth_packet.set_source(interface.get_mac().into());
        eth_packet.set_ethertype(self.ethertype);

        let mut arp_buf = vec![0; 28];
        let mut arp_packet = MutableArpPacket::new(&mut arp_buf).unwrap();

        arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
        arp_packet.set_protocol_type(EtherTypes::Ipv4);
        arp_packet.set_hw_addr_len(0x06);
        arp_packet.set_proto_addr_len(0x04);
        arp_packet.set_operation(ArpOperation::new(self.operation as u16));
        arp_packet.set_sender_hw_addr(self.source_hardware_address.into());
        arp_packet.set_sender_proto_addr(self.source_protocol_address);
        arp_packet.set_target_hw_addr(self.target_hardware_address.into());
        arp_packet.set_target_proto_addr(self.target_protocol_address);

        eth_packet.set_payload(arp_packet.packet_mut());

        tx.send_to(eth_packet.packet(), None).unwrap()
    }
}

impl TryFrom<ArpPacket<'_>> for ArpMessage {
    type Error = Error;

    fn try_from(arp_packet: ArpPacket<'_>) -> Result<Self, Self::Error> {
        let operation_raw = arp_packet.get_operation().0;
        let operation = match FromPrimitive::from_u16(operation_raw) {
            Some(op) => op,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Could not cast operation raw value {} to enum.",
                        operation_raw
                    ),
                ))
            }
        };

        Ok(ArpMessage::new(
            arp_packet.get_protocol_type(),
            arp_packet.get_sender_hw_addr().into(),
            arp_packet.get_sender_proto_addr(),
            arp_packet.get_target_hw_addr().into(),
            arp_packet.get_target_proto_addr(),
            operation,
        ))
    }
}
