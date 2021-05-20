use pnet::{datalink::{NetworkInterface, interfaces}};


pub fn get_all_interfaces() -> Vec<NetworkInterface> {
    interfaces()
}

pub fn get_interface_by_guess(interfaces: &Vec<NetworkInterface>) -> Option<&NetworkInterface> {
    let considered_ifaces = interfaces
        .iter()
        .filter(|iface| !iface.is_loopback() && iface.is_up() && !iface.ips.is_empty())
        .collect::<Vec<&NetworkInterface>>();

    match considered_ifaces.first() {
        Some(iface) => Some(*iface),
        None => None
    }
}

pub fn get_interface_by_name<'a>(interfaces: &'a Vec<NetworkInterface>, name: &str) -> Option<&'a NetworkInterface> {
    let considered_ifaces = interfaces
        .iter()
        .filter(|iface| iface.name == *name)
        .collect::<Vec<&NetworkInterface>>();

    match considered_ifaces.first() {
        Some(iface) => Some(*iface),
        None => None
    }
}