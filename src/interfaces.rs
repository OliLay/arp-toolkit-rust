use pnet::datalink::{interfaces, NetworkInterface};
use std::net::{IpAddr, Ipv4Addr};

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

    pub fn new_by_name(interface_name: &str) -> Option<Self> {
        let iface = Interface::get_interface_by_name(&interface_name);

        match iface {
            Some(_) => Some(Interface {
                network_interface: iface,
            }),
            None => None,
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
        self.network_interface.as_ref().unwrap().mac.unwrap().into()
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

    fn get_interface_by_name(name: &str) -> Option<NetworkInterface> {
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

// redefinition, so that as a user pnet does not need to be imported
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct MacAddr(pub u8, pub u8, pub u8, pub u8, pub u8, pub u8);

impl MacAddr {
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Self {
        MacAddr(a, b, c, d, e, f)
    }
}

impl std::fmt::Display for MacAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0, self.1, self.2, self.3, self.4, self.5
        )
    }
}

impl From<pnet::util::MacAddr> for MacAddr {
    fn from(pnet_mac_addr: pnet::util::MacAddr) -> Self {
        MacAddr(
            pnet_mac_addr.0,
            pnet_mac_addr.1,
            pnet_mac_addr.2,
            pnet_mac_addr.3,
            pnet_mac_addr.4,
            pnet_mac_addr.5,
        )
    }
}

impl From<MacAddr> for pnet::util::MacAddr {
    fn from(mac_addr: MacAddr) -> Self {
        pnet::util::MacAddr(
            mac_addr.0, mac_addr.1, mac_addr.2, mac_addr.3, mac_addr.4, mac_addr.5,
        )
    }
}
