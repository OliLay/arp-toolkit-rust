# arp-toolkit
![crates.io](https://img.shields.io/crates/v/arp-toolkit.svg)

Toolkit for (R)ARP , the `(Reverse) Address Resolution Protocol`.

## Build
To build it, use
```
cargo build
```

## Usage
The `example/example.rs` file shows high-level usage of the class `ArpClient`. For example, it allows to use this toolkit to get the MAC address for the corresponding IP and vice versa with a single function call. 

It is also possible to directly send and receive (R)ARP packages. To see how this works, take a look at `lib/arp.rs`

**Note**: As this library uses Raw sockets, either [set](https://squidarth.com/networking/systems/rc/2018/05/28/using-raw-sockets.html) the `CAP_NET_RAW` capability or run your program with `sudo`. (not recommended)

## Development
- [x] Basic ARP functionality implemented ((R)ARP requests/responses)
- [x] Sync client (with timeouts)
- [ ] Async client
- [ ] Server