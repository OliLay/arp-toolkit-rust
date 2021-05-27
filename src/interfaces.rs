use std::net::{IpAddr, Ipv4Addr};

use pnet::{datalink::{interfaces, NetworkInterface}, util::MacAddr};

pub struct Interface {
    interfaces: Vec<NetworkInterface>,
    network_interface: Option<NetworkInterface>,
}

impl Interface {
    pub fn new() -> Self {
        Interface {
            interfaces: Interface::get_all_interfaces(),
            network_interface: None
        }
    }

 //   pub fn new_from_name(name: String) -> Self {
   //     let interfaces = Interface::get_all_interfaces();

     //   Interface {
       //     network_interface: Interface::get_interface_by_name(interfaces, &name),
       // }
   // }

    fn assign_interface(&'static mut self) {
        let iface = self.assign_interface_by_guess();
        self.network_interface = iface;
    }
        
    pub fn get_ip(&self) -> Ipv4Addr {
        self.network_interface.unwrap()
            .ips
            .iter()
            .find(|ip| ip.is_ipv4())
            .map(|ip| match ip.ip() {
                IpAddr::V4(ip) => ip,
                _ => panic!("Interface only supports IPv6, but IPv4 address was requested."),
            })
            .unwrap()
    }

    pub fn get_mac(&self) -> MacAddr {
        self.network_interface.unwrap().mac.unwrap()
    }

    pub fn get_raw_interface(&self) -> &NetworkInterface {
        &self.network_interface.unwrap()
    }

    fn get_all_interfaces() -> Vec<NetworkInterface> {
        interfaces()
    }

    fn assign_interface_by_guess(&self) -> Option<NetworkInterface> {
        let considered_ifaces = self.interfaces
            .iter()
            .filter(|iface| !iface.is_loopback() && iface.is_up() && !iface.ips.is_empty())
            .collect::<Vec<&NetworkInterface>>();

        match considered_ifaces.first() {
            Some(iface) => Some(**iface),
            None => None,
        }
    }

    fn get_interface_by_name(
        interfaces: Vec<NetworkInterface>,
        name: &str,
    ) -> Option<NetworkInterface> {
        let considered_ifaces = interfaces
            .iter()
            .filter(|iface| iface.name == *name)
            .collect::<Vec<&NetworkInterface>>();

        match considered_ifaces.first() {
            Some(iface) => Some(**iface),
            None => None,
        }
    }
}
