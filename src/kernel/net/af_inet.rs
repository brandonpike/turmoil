use std::io::{Error, ErrorKind, Result};
use std::ops::RangeInclusive;

use crate::kernel::core::socket::{Socket, SocketProtocol};
use crate::kernel::net::udp;

pub struct AfInet {
    udp_bus: udp::UdpBus,
}

impl AfInet {
    pub fn new(port_range: RangeInclusive<u64>) -> Self {
        Self {
            udp_bus: udp::UdpBus::new(port_range),
        }
    }

    pub fn bind(&mut self, sock: &mut Socket) -> Result<()> {
        let addr = sock.addr;
        // Right now we only support 0.0.0.0 or localhost.
        if !addr.ip().is_unspecified() && !addr.ip().is_loopback() {
            return Err(Error::new(
                ErrorKind::AddrNotAvailable,
                format!("{addr} is not supported"),
            ));
        }

        if addr.port() == 0 {
            let port = match sock.sock_protocol {
                SocketProtocol::Tcp => todo!(),
                SocketProtocol::Udp => self.udp_bus.ephemeral_port()?,
            };
            sock.addr.set_port(port);
        }

        match sock.sock_protocol {
            SocketProtocol::Tcp => todo!(),
            SocketProtocol::Udp => self.udp_bus.bind(sock),
        }
    }

    pub fn unbind(&mut self, sock: &mut Socket) -> Result<()> {
        match sock.sock_protocol {
            SocketProtocol::Tcp => todo!(),
            SocketProtocol::Udp => self.udp_bus.unbind(sock),
        }
    }
}
