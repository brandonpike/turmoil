use crate::{Config, Io, Message, Rt, ToSocketAddr, World};

use indexmap::IndexMap;
use std::cell::RefCell;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{Duration, Instant};

/// Network simulation
pub struct Sim {
    /// Simulation configuration
    config: Config,

    /// Tracks the simulated world state.
    ///
    /// This is what is stored in the thread-local
    world: RefCell<World>,

    /// Per simulated host Tokio runtimes
    ///
    /// When `None`, this signifies the host is a "client"
    rts: IndexMap<SocketAddr, Option<Rt>>,
}

impl Sim {
    pub(crate) fn new(config: Config, world: World) -> Sim {
        Sim {
            config,
            world: RefCell::new(world),
            rts: IndexMap::new(),
        }
    }

    /// Register a host with the simulation
    pub fn register<F, M, R>(&mut self, addr: impl ToSocketAddr, host: F)
    where
        F: FnOnce(Io<M>) -> R,
        M: Message,
        R: Future<Output = ()> + 'static,
    {
        let rt = Rt::new();
        let epoch = rt.now();
        let addr = self.lookup(addr);

        let io = {
            let world = RefCell::get_mut(&mut self.world);
            let notify = Arc::new(Notify::new());

            // Register host state with the world
            world.register(addr, epoch, notify.clone());

            Io::new(addr, notify)
        };

        World::enter(&self.world, || {
            rt.with(|| {
                tokio::task::spawn_local(host(io));
            });
        });

        self.rts.insert(addr, Some(rt));
    }

    /// Lookup a socket address by host name
    pub fn lookup(&self, addr: impl ToSocketAddr) -> SocketAddr {
        self.world.borrow_mut().lookup(addr)
    }

    // Introduce a full network partition between two hosts
    pub fn partition(&self, a: impl ToSocketAddr, b: impl ToSocketAddr) {
        let mut world = self.world.borrow_mut();
        let a = world.lookup(a);
        let b = world.lookup(b);

        world.partition(a, b);
    }

    // Repair a partition between two hosts
    pub fn repair(&self, a: impl ToSocketAddr, b: impl ToSocketAddr) {
        let mut world = self.world.borrow_mut();
        let a = world.lookup(a);
        let b = world.lookup(b);

        world.repair(a, b);
    }

    /// Set the max message latency
    pub fn set_max_message_latency(&self, value: Duration) {
        self.world
            .borrow_mut()
            .topology
            .set_max_message_latency(value);
    }

    pub fn set_link_max_message_latency(
        &self,
        a: impl ToSocketAddr,
        b: impl ToSocketAddr,
        value: Duration,
    ) {
        let mut world = self.world.borrow_mut();
        let a = world.lookup(a);
        let b = world.lookup(b);

        world.topology.set_link_max_message_latency(a, b, value);
    }

    /// Set the message latency distribution curve.
    ///
    /// Message latency follows an exponential distribution curve. The `value`
    /// is the lambda argument to the probability function.
    pub fn set_message_latency_curve(&self, value: f64) {
        self.world
            .borrow_mut()
            .topology
            .set_message_latency_curve(value);
    }

    pub fn set_fail_rate(&mut self, value: f64) {
        self.world.borrow_mut().topology.set_fail_rate(value);
    }

    pub fn set_link_fail_rate(&mut self, a: impl ToSocketAddr, b: impl ToSocketAddr, value: f64) {
        let mut world = self.world.borrow_mut();
        let a = world.lookup(a);
        let b = world.lookup(b);

        world.topology.set_link_fail_rate(a, b, value);
    }

    /// Create a client handle
    pub fn client<M: Message>(&mut self, addr: impl ToSocketAddr) -> Io<M> {
        let world = RefCell::get_mut(&mut self.world);
        let addr = world.lookup(addr);
        let notify = Arc::new(Notify::new());

        world.register(addr, Instant::now(), notify.clone());
        self.rts.insert(addr, None);

        Io::new(addr, notify)
    }

    pub fn run_until<F>(&mut self, until: F)
    where
        F: Future<Output = ()> + 'static,
    {
        let rt = Rt::new();
        let task = World::enter(&self.world, || rt.with(|| tokio::task::spawn_local(until)));

        let mut elapsed = Duration::default();
        let tick = self.config.tick;

        loop {
            World::enter(&self.world, || rt.tick(tick));

            if task.is_finished() {
                return;
            }

            for (&addr, maybe_rt) in self.rts.iter() {
                if let Some(rt) = maybe_rt {
                    // Set the current host
                    self.world.borrow_mut().current = Some(addr);

                    let now = World::enter(&self.world, || rt.tick(tick));

                    // Unset the current host
                    self.world.borrow_mut().current = None;

                    let mut world = self.world.borrow_mut();
                    world.tick(addr, now);
                } else {
                    let mut world = self.world.borrow_mut();
                    let now = world.host(addr).now;
                    world.tick(addr, now + tick);
                }
            }

            elapsed += tick;

            if elapsed > self.config.duration {
                panic!("Ran for {:?} without completing", self.config.duration);
            }
        }
    }
}