pub mod core;
pub mod net;

use core::{
    descriptor::DescriptorGuardManager,
    socket::{Socket, SocketFdManager, SocketProtocol},
};
use net::af_inet::{self, AfInet};

use std::{
    io::{Error, ErrorKind, Result},
    net::{IpAddr, SocketAddr},
};

use crate::Datagram;

pub struct Kernel {
    af_inet: af_inet::AfInet,
    sock_fd_manager: SocketFdManager,

    // FIXME: kernels don't have addresses... this needs to be replaced with a
    // device registry.
    addr: IpAddr,
}

impl Kernel {
    pub(crate) fn new(addr: IpAddr) -> Self {
        let sock_fd_manager = DescriptorGuardManager::new(1..=65535);

        Self {
            af_inet: AfInet::new(1..=65535),
            sock_fd_manager,
            addr,
        }
    }

    pub fn bind(&mut self, addr: SocketAddr, sock_protocol: SocketProtocol) -> Result<Socket> {
        if addr.is_ipv4() != self.addr.is_ipv4() {
            panic!("ip version mismatch: {:?} host: {:?}", addr, self.addr)
        }

        let _sock_fd_guard = self.sock_fd_manager.ephemeral().ok_or(Error::new(
            ErrorKind::Other,
            "ran out of descriptors - try raising the limits",
        ))?;
        let sock_fd = *_sock_fd_guard;
        let mut sock = Socket::new(addr, sock_fd, sock_protocol, _sock_fd_guard);

        self.af_inet.bind(&mut sock)?;

        Ok(sock)
    }

    pub fn unbind(&mut self, sock: &mut Socket) -> Result<()> {
        self.af_inet.unbind(sock)
    }

    pub fn send_to(&mut self, datagram: Datagram, target: SocketAddr) -> Result<usize> {
        Ok(0)
    }
}
