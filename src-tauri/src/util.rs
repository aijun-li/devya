use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

pub fn is_port_free(port: u16) -> bool {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    TcpListener::bind(addr).is_ok()
}
