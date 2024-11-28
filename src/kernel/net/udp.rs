use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::sync::Arc;

use crate::kernel::core::descriptor::{DescriptorGuard, DescriptorGuardManager};
use crate::kernel::core::socket::{SockFd, Socket};

type Ports = Arc<DescriptorGuardManager>;

#[derive(Hash, Eq, PartialEq)]
struct Bind {
    addr: SocketAddr,
    _port_guard: DescriptorGuard,
}

pub struct UdpBus {
    binds: HashMap<SockFd, Bind>,
    ports: Ports,
}

impl UdpBus {
    pub fn new(port_range: RangeInclusive<u64>) -> Self {
        let ports = DescriptorGuardManager::new(port_range);

        Self {
            binds: HashMap::new(),
            ports,
        }
    }

    pub fn bind(&mut self, sock: &mut Socket) -> Result<()> {
        let _port_guard = self
            .ports
            .register(sock.addr.port().into())
            .ok_or(Error::new(ErrorKind::AddrInUse, "address in use"))?;

        let bind = Bind {
            addr: sock.addr.clone(),
            _port_guard,
        };

        self.binds.insert(sock.sock_fd, bind);

        Ok(())
    }

    pub fn unbind(&mut self, sock: &mut Socket) -> Result<()> {
        let bind = self.binds.remove(&sock.sock_fd);
        // Release the address back to the kernel
        drop(bind);

        Ok(())
    }

    pub fn ephemeral_port(&mut self) -> Result<u16> {
        let port = self.ports.ephemeral().ok_or(Error::new(
            ErrorKind::AddrNotAvailable,
            "ran out of ephemeral ports",
        ))?;

        Ok(u16::try_from(*port).unwrap())
    }
}
