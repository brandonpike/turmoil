use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::{Deref, RangeInclusive};
use std::sync::{Arc, Mutex};

pub type Fd = u64;
pub type Manager = Arc<DescriptorGuardManager>;

#[derive(Clone)]
pub struct DescriptorGuard {
    id: Fd,
    manager: Manager,
}

impl DescriptorGuard {
    pub fn new(id: Fd, manager: Manager) -> Self {
        Self { id, manager }
    }
}

impl Deref for DescriptorGuard {
    type Target = Fd;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl Hash for DescriptorGuard {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.id);
    }
}

impl Borrow<Fd> for DescriptorGuard {
    fn borrow(&self) -> &Fd {
        &self.id
    }
}

impl Eq for DescriptorGuard {}

impl PartialEq for DescriptorGuard {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Drop for DescriptorGuard {
    fn drop(&mut self) {
        self.manager.release(self.clone());
    }
}

pub struct DescriptorGuardManager {
    manager: Mutex<HashSet<DescriptorGuard>>,
}

impl DescriptorGuardManager {
    pub fn new(range: RangeInclusive<Fd>) -> Arc<Self> {
        let set = HashSet::new();
        let manager = Arc::new(Self {
            manager: Mutex::new(set),
        });

        {
            let mut set = manager.manager.lock().unwrap();
            for id in range {
                let p = DescriptorGuard::new(id, manager.clone());
                set.insert(p);
            }
        }

        manager
    }

    pub fn register(&self, id: Fd) -> Option<DescriptorGuard> {
        let mut set = self.manager.lock().unwrap();

        let id = set.get(&id).cloned();

        if let Some(ref id) = id {
            set.remove(id);
        }

        id
    }

    pub fn ephemeral(&self) -> Option<DescriptorGuard> {
        let mut set = self.manager.lock().unwrap();
        let id = set.iter().next().cloned();

        if let Some(ref id) = id {
            set.remove(id);
        }

        id
    }

    fn release(&self, id: DescriptorGuard) {
        self.manager.lock().unwrap().insert(id);
    }
}

#[cfg(test)]
mod tests {
    use super::DescriptorGuardManager;

    #[cfg(test)]
    fn ephemeral_test() {
        let fd_manager = DescriptorGuardManager::new(1..=2);

        let fd_guard_1 = fd_manager.ephemeral().unwrap();
        let fd_guard_2 = fd_manager.ephemeral().unwrap();

        assert_ne!(*fd_guard_1, *fd_guard_2);
    }

    #[cfg(test)]
    fn overlap_test() {
        let fd_manager = DescriptorGuardManager::new(1..=5);

        assert!(fd_manager.register(1).is_some());
        assert!(fd_manager.register(1).is_none());
    }
}
