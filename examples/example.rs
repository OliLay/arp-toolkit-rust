use lib::client::ArpClient;
use std::net::Ipv4Addr;
use pnet::util::MacAddr;

fn main() {
    let searched_mac = MacAddr::new(0xdc, 0xa6, 0x32, 0x27, 0x5b, 0xd8);

    let mut client = ArpClient::new();

    let result = client.mac_to_ip(searched_mac, None);
    println!("le answer #1 is {}", result.unwrap());

    let result = client.ip_to_mac(Ipv4Addr::new(10, 0, 0, 2), None);
    println!("le answer #2 is {}", result.unwrap());
}