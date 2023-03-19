use pnet::datalink::{
    channel, interfaces, Channel, DataLinkReceiver, DataLinkSender, NetworkInterface,
};
use std::{
    io::{Error, ErrorKind},
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

/// Represents a network interface.
/// Wraps pnet's `NetworkInterface` struct for better convenience.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Interface {
    network_interface: NetworkInterface,
}

impl Interface {
    /// Selects the first "best-suited" interface found.
    pub fn new() -> Result<Self, Error> {
        match Interface::get_interface_by_guess() {
            Some(iface) => Ok(Interface {
                network_interface: iface,
            }),
            None => Err(Error::new(
                ErrorKind::NotConnected,
                "Could not get any network interface.",
            )),
        }
    }

    /// Selects the interface with the name `interface_name`.
    pub fn new_by_name(interface_name: &str) -> Option<Self> {
        let iface = Interface::get_interface_by_name(&interface_name);

        match iface {
            Some(iface) => Some(Interface {
                network_interface: iface,
            }),
            None => None,
        }
    }

    /// Returns the IPv4 address of the interface.
    pub fn get_ip(&self) -> Result<Ipv4Addr, Error> {
        let ip = self
            .network_interface
            .ips
            .iter()
            .find(|ip| ip.is_ipv4())
            .map(|ip| match ip.ip() {
                IpAddr::V4(ip) => Some(ip),
                _ => None,
            })
            .unwrap_or(None);

        match ip {
            Some(ip) => Ok(ip),
            None => Err(Error::new(
                ErrorKind::AddrNotAvailable,
                "Currently selected interface does not have any IP address assigned.",
            )),
        }
    }

    /// Returns the MAC address assigned to the interface.
    pub fn get_mac(&self) -> Result<MacAddr, Error> {
        match self.network_interface.mac {
            Some(mac) => Ok(mac.into()),
            None => Err(Error::new(
                ErrorKind::AddrNotAvailable,
                "Currently selected interface does not have any MAC address assigned.",
            )),
        }
    }

    /// Returns the raw `pnet` interface instance related to the interface.
    pub fn get_raw_interface(&self) -> &NetworkInterface {
        &self.network_interface
    }

    /// Creates and returns a new Ethernet (tx, rx) channel pair on the interface.
    pub fn create_tx_rx_channels(
        &self,
    ) -> Result<(Box<dyn DataLinkSender>, Box<dyn DataLinkReceiver>), Error> {
        let channel_config = pnet::datalink::Config {
            read_timeout: Some(Duration::ZERO),
            ..Default::default()
        };

        match channel(&self.get_raw_interface(), channel_config) {
            Ok(Channel::Ethernet(tx, rx)) => Ok((tx, rx)),
            Ok(_) => return Err(Error::new(ErrorKind::Other, "Unknown channel type")),
            Err(err) => return Err(err),
        }
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

/// Redefinition of the pnet `MacAddr`, so that as a user pnet does not need to be imported
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
