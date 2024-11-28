use std::net::SocketAddr;
use std::sync::Arc;

use crate::kernel::core::descriptor::{DescriptorGuard, DescriptorGuardManager, Fd};

pub type SockFd = Fd;
pub type SocketFdManager = Arc<DescriptorGuardManager>;

pub enum SocketDomain {
    AfInet,
}

#[derive(Clone)]
pub enum SocketProtocol {
    Tcp,
    Udp,
}

pub struct Socket {
    pub addr: SocketAddr,
    pub sock_fd: SockFd,
    pub sock_protocol: SocketProtocol,
    _sock_fd_guard: DescriptorGuard,
}

impl Socket {
    pub fn new(
        addr: SocketAddr,
        sock_fd: SockFd,
        sock_protocol: SocketProtocol,
        _sock_fd_guard: DescriptorGuard,
    ) -> Self {
        Socket {
            addr,
            sock_fd,
            sock_protocol,
            _sock_fd_guard,
        }
    }
}
