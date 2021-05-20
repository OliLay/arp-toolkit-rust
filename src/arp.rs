use std::{io::Error, net::Ipv4Addr, u16};

use pnet::{
    datalink::{channel, Channel, NetworkInterface},
    packet::{
        arp::{ArpHardwareTypes, ArpOperation, MutableArpPacket},
        ethernet::{
            EtherType,
            EtherTypes::{self},
            MutableEthernetPacket,
        },
        MutablePacket, Packet,
    },
    util::MacAddr,
};
pub struct ArpMessage {
    source_hardware_address: MacAddr,
    source_protocol_address: Ipv4Addr,

    target_hardware_address: MacAddr,
    target_protocol_address: Ipv4Addr,

    ethertype: EtherType,
    operation: Operation,
}

#[derive(Copy, Clone)]
enum Operation {
    ArpRequest = 0x1,
    ArpResponse = 0x2,
    RarpRequest = 0x3,
    RarpResponse = 0x4,
}

impl ArpMessage {
    fn new(
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

    pub fn send(&self, interface: &NetworkInterface) -> Result<(), Error> {
        let mut tx = match channel(&interface, Default::default()) {
            Ok(Channel::Ethernet(tx, _)) => tx,
            Ok(_) => panic!("Unknown channel type"),
            Err(err) => panic!("Error when opening channel: {}", err),
        };

        let mut eth_buf = vec![0; 42];
        let mut eth_packet = MutableEthernetPacket::new(&mut eth_buf).unwrap();

        eth_packet.set_destination(MacAddr::broadcast());
        eth_packet.set_source(interface.mac.unwrap());
        eth_packet.set_ethertype(self.ethertype);

        let mut rarp_buf = vec![0; 28];
        let mut rarp_packet = MutableArpPacket::new(&mut rarp_buf).unwrap();

        rarp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
        rarp_packet.set_protocol_type(EtherTypes::Ipv4);
        rarp_packet.set_hw_addr_len(0x06);
        rarp_packet.set_proto_addr_len(0x04);
        rarp_packet.set_operation(ArpOperation::new(self.operation as u16));
        rarp_packet.set_sender_hw_addr(self.source_hardware_address);
        rarp_packet.set_sender_proto_addr(self.source_protocol_address);
        rarp_packet.set_target_hw_addr(self.target_hardware_address);
        rarp_packet.set_target_proto_addr(self.target_protocol_address);

        eth_packet.set_payload(rarp_packet.packet_mut());

        tx.send_to(eth_packet.packet(), None).unwrap()
    }
}
