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
    pub fn new(master_lock_id: u32) -> Locker {
        Locker {
            locks: Vec::new(),
            master_lock_id,
        }
    }

    /// Adds an interface to be managed by the locker.
    ///
    /// If the interface is already present, it does nothing.
    ///
    /// # Arguments
    ///
    /// * `interface_id` - The ID of the interface to add.
    pub fn add_interface(&mut self, interface_id: usize) {
        if self.get_interface_index(interface_id).is_none() {
            self.locks
                .push(Lock {
                    status: LockStatus::Unlocked,
                    interface_id,
                })
                .unwrap();
        }
    }

    fn get_interface_index(&self, interface_id: usize) -> Option<usize> {
        for (i, lock) in self.locks.iter().enumerate() {
            if lock.interface_id == interface_id {
                return Some(i);
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
    pub fn lock_interface(&mut self, interface_id: usize, locker_id: u32) -> HalResult<()> {
        if let Some(index) = self.get_interface_index(interface_id) {
            match &self.locks[index].status {
                LockStatus::Locked(lock_id) => {
                    if *lock_id == locker_id {
                        Ok(())
                    } else if locker_id == self.master_lock_id {
                        self.locks[index].status = LockStatus::Locked(locker_id);
                        Ok(())
                    } else {
                        Err(crate::HalError::LockedInterface(interface_name(
                            interface_id,
                        )?))
                    }
                }
                LockStatus::Unlocked => {
                    self.locks[index].status = LockStatus::Locked(locker_id);
                    Ok(())
                }
            }
        } else {
            Err(crate::HalError::WrongInterfaceId(interface_id))
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
    pub fn unlock_interface(&mut self, interface_id: usize, locker_id: u32) -> HalResult<()> {
        if let Some(index) = self.get_interface_index(interface_id) {
            match &self.locks[index].status {
                LockStatus::Locked(lock_id) => {
                    if *lock_id == locker_id || locker_id == self.master_lock_id {
                        self.locks[index].status = LockStatus::Unlocked;
                        Ok(())
                    } else {
                        Err(crate::HalError::InterfaceAlreadyLocked(interface_name(
                            interface_id,
                        )?))
                    }
                }
                LockStatus::Unlocked => Ok(()),
            }
        } else {
            Err(crate::HalError::WrongInterfaceId(interface_id))
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
    pub fn authorize_action(&self, interface_id: usize, caller_id: u32) -> HalResult<()> {
        if let Some(index) = self.get_interface_index(interface_id) {
            match &self.locks[index].status {
                LockStatus::Locked(locker_id) => {
                    if *locker_id == caller_id {
                        Ok(())
                    } else {
                        Err(crate::HalError::LockedInterface(interface_name(
                            interface_id,
                        )?))
                    }
                }
                LockStatus::Unlocked => Ok(()),
            }
        } else {
            Err(crate::HalError::WrongInterfaceId(interface_id))
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
    pub fn is_locked(&self, interface_id: usize) -> HalResult<Option<u32>> {
        if let Some(index) = self.get_interface_index(interface_id) {
            match &self.locks[index].status {
                LockStatus::Locked(owner) => Ok(Some(*owner)),
                LockStatus::Unlocked => Ok(None),
            }
        } else {
            Err(crate::HalError::WrongInterfaceId(interface_id))
        }
    }
}
