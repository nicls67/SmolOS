use crate::HalResult;
use crate::bindings::interface_name;
use heapless::Vec;

#[derive(Debug)]
enum LockStatus {
    Locked(u32),
    Unlocked,
}

#[derive(Debug)]
struct Lock {
    status: LockStatus,
    interface_id: usize,
}

/// A structure to manage locks on hardware interfaces.
///
/// The `Locker` holds the status of multiple locks, identified by their interface ID.
/// It allows locking and unlocking interfaces based on a `locker_id`.
/// A master lock ID is provided at creation to override locks.
pub struct Locker {
    locks: Vec<Lock, 64>,
    master_lock_id: u32,
}

impl Locker {
    /// Creates a new `Locker`.
    ///
    /// # Arguments
    ///
    /// * `master_lock_id` - An identifier that has master privileges to lock/unlock any interface.
    pub fn new(p_master_lock_id: u32) -> Locker {
        Locker {
            locks: Vec::new(),
            master_lock_id: p_master_lock_id,
        }
    }

    /// Adds an interface to be managed by the locker.
    ///
    /// If the interface is already present, it does nothing.
    ///
    /// # Arguments
    ///
    /// * `interface_id` - The ID of the interface to add.
    pub fn add_interface(&mut self, p_interface_id: usize) {
        if self.get_interface_index(p_interface_id).is_none() {
            self.locks
                .push(Lock {
                    status: LockStatus::Unlocked,
                    interface_id: p_interface_id,
                })
                .unwrap();
        }
    }

    fn get_interface_index(&self, p_interface_id: usize) -> Option<usize> {
        for (l_i, l_lock) in self.locks.iter().enumerate() {
            if l_lock.interface_id == p_interface_id {
                return Some(l_i);
            }
        }
        None
    }

    /// Locks an interface for a specific locker ID.
    ///
    /// # Arguments
    ///
    /// * `interface_id` - The ID of the interface to lock.
    /// * `locker_id` - The ID of the entity requesting the lock.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the lock was successful or if the interface was already locked by the same ID.
    /// * `Err(HalError::LockedInterface)` if the interface is already locked by another ID and the requester is not the master.
    /// * `Err(HalError::WrongInterfaceId)` if the interface ID is not managed by this locker.
    pub fn lock_interface(&mut self, p_interface_id: usize, p_locker_id: u32) -> HalResult<()> {
        if let Some(l_index) = self.get_interface_index(p_interface_id) {
            match &self.locks[l_index].status {
                LockStatus::Locked(l_lock_id) => {
                    if *l_lock_id == p_locker_id {
                        Ok(())
                    } else if p_locker_id == self.master_lock_id {
                        self.locks[l_index].status = LockStatus::Locked(p_locker_id);
                        Ok(())
                    } else {
                        Err(crate::HalError::LockedInterface(interface_name(
                            p_interface_id,
                        )?))
                    }
                }
                LockStatus::Unlocked => {
                    self.locks[l_index].status = LockStatus::Locked(p_locker_id);
                    Ok(())
                }
            }
        } else {
            Err(crate::HalError::WrongInterfaceId(p_interface_id))
        }
    }

    /// Unlocks an interface.
    ///
    /// # Arguments
    ///
    /// * `interface_id` - The ID of the interface to unlock.
    /// * `locker_id` - The ID of the entity requesting the unlock.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the unlock was successful or if the interface was already unlocked.
    /// * `Err(HalError::InterfaceAlreadyLocked)` if the interface is locked by another ID and the requester is not the master.
    /// * `Err(HalError::WrongInterfaceId)` if the interface ID is not managed by this locker.
    pub fn unlock_interface(&mut self, p_interface_id: usize, p_locker_id: u32) -> HalResult<()> {
        if let Some(l_index) = self.get_interface_index(p_interface_id) {
            match &self.locks[l_index].status {
                LockStatus::Locked(l_lock_id) => {
                    if *l_lock_id == p_locker_id || p_locker_id == self.master_lock_id {
                        self.locks[l_index].status = LockStatus::Unlocked;
                        Ok(())
                    } else {
                        Err(crate::HalError::InterfaceAlreadyLocked(interface_name(
                            p_interface_id,
                        )?))
                    }
                }
                LockStatus::Unlocked => Ok(()),
            }
        } else {
            Err(crate::HalError::WrongInterfaceId(p_interface_id))
        }
    }

    /// Checks if an action is authorized for a given caller on a specific interface.
    ///
    /// An action is authorized if the interface is unlocked, or if it is locked by the `caller_id`.
    ///
    /// # Arguments
    ///
    /// * `interface_id` - The ID of the interface to check.
    /// * `caller_id` - The ID of the entity attempting the action.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the action is authorized.
    /// * `Err(HalError::LockedInterface)` if the interface is locked by another ID.
    /// * `Err(HalError::WrongInterfaceId)` if the interface ID is not managed by this locker.
    pub fn authorize_action(&self, p_interface_id: usize, p_caller_id: u32) -> HalResult<()> {
        if let Some(l_index) = self.get_interface_index(p_interface_id) {
            match &self.locks[l_index].status {
                LockStatus::Locked(l_locker_id) => {
                    if *l_locker_id == p_caller_id {
                        Ok(())
                    } else {
                        Err(crate::HalError::LockedInterface(interface_name(
                            p_interface_id,
                        )?))
                    }
                }
                LockStatus::Unlocked => Ok(()),
            }
        } else {
            Err(crate::HalError::WrongInterfaceId(p_interface_id))
        }
    }

    /// Checks whether an interface is currently locked.
    ///
    /// # Arguments
    ///
    /// * `interface_id` - The ID of the interface to query.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(locker_id))` if the interface is locked, where `locker_id` is the ID of the lock owner.
    /// * `Ok(None)` if the interface is unlocked.
    /// * `Err(HalError::WrongInterfaceId)` if the interface ID is not managed by this locker.
    pub fn is_locked(&self, p_interface_id: usize) -> HalResult<Option<u32>> {
        if let Some(l_index) = self.get_interface_index(p_interface_id) {
            match &self.locks[l_index].status {
                LockStatus::Locked(l_owner) => Ok(Some(*l_owner)),
                LockStatus::Unlocked => Ok(None),
            }
        } else {
            Err(crate::HalError::WrongInterfaceId(p_interface_id))
        }
    }
}
