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

pub struct Locker {
    locks: Vec<Lock, 256>,
    master_lock_id: u32,
}

impl Locker {
    pub fn new(master_lock_id: u32) -> Locker {
        Locker {
            locks: Vec::new(),
            master_lock_id,
        }
    }

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
}
