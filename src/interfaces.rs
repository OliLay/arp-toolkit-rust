use std::net::{IpAddr, Ipv4Addr};

use pnet::{
    datalink::{interfaces, NetworkInterface},
    util::MacAddr,
};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Interface {
    network_interface: Option<NetworkInterface>,
}

impl Interface {
    pub fn new() -> Self {
        Interface {
            network_interface: Interface::get_interface_by_guess(),
        }
    }

    pub fn get_ip(&self) -> Ipv4Addr {
        self.network_interface
            .as_ref()
            .unwrap()
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
        self.network_interface.as_ref().unwrap().mac.unwrap()
    }

    pub fn get_raw_interface(&self) -> &NetworkInterface {
        &self.network_interface.as_ref().unwrap()
    }

    fn get_all_interfaces() -> Vec<NetworkInterface> {
        interfaces()
    }

    fn get_interface_by_guess() -> Option<NetworkInterface> {
        let considered_ifaces = Interface::get_all_interfaces()
            .into_iter()
            .filter(|iface| !iface.is_loopback() && iface.is_up() && !iface.ips.is_empty())
            .collect::<Vec<NetworkInterface>>();

        match considered_ifaces.first() {
            Some(iface) => Some(iface.clone()),
            None => None,
        }
    }

    
    fn get_interface_by_name(&self, name: &str) -> Option<NetworkInterface> {
        let considered_ifaces = Interface::get_all_interfaces()
            .into_iter()
            .filter(|iface| iface.name == *name)
            .collect::<Vec<NetworkInterface>>();

        match considered_ifaces.first() {
            Some(iface) => Some(iface.clone()),
            None => None,
        }
    }
    
}
