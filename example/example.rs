extern crate libarp;

use libarp::{client::ArpClient, interfaces::MacAddr};
use std::net::Ipv4Addr;

fn main() {
    let mut client = ArpClient::new();

    let result = client.mac_to_ip(MacAddr::new(0xdc, 0xa6, 0x32, 0x27, 0x5b, 0xd8), None);
    println!("IP is {}", result.unwrap());

    let result = client.ip_to_mac(Ipv4Addr::new(10, 0, 0, 2), None);
    println!("MAC is {}", result.unwrap());
}
