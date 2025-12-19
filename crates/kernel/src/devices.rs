use crate::{KernelError, KernelResult, data::Kernel, ident::KERNEL_MASTER_ID};

/// Device locking and authorization utilities.
///
/// This module defines:
/// - [`DeviceType`], an identifier for lockable devices (terminal, display, or HAL peripherals).
/// - [`LockState`], a simple lock ownership state (`Locked(owner_id)` / `Unlocked`).
/// - [`DevicesManager`], which tracks lock state for built-in devices (terminal/display) and
///   delegates peripheral lock management to the HAL.
///
/// Lock ownership is represented by a caller identifier (`caller_id: u32`). The
/// [`KERNEL_MASTER_ID`] is treated as a privileged owner that can take over (lock) and release
/// (unlock) devices regardless of current ownership.
pub enum DeviceType {
    /// The system terminal device.
    Terminal,
    /// The system display device.
    Display,
    /// A HAL-defined peripheral/interface by numeric identifier.
    Peripheral(usize),
}

impl DeviceType {
    /// Returns a human-readable name for this device.
    ///
    /// # Returns
    /// - `Ok(&'static str)` containing the device name.
    ///
    /// # Errors
    /// - For [`DeviceType::Peripheral`], returns `Err(KernelError::HalError(_))` if the HAL cannot
    ///   resolve the interface name.
    pub fn name(&self) -> KernelResult<&'static str> {
        match self {
            DeviceType::Terminal => Ok("Terminal"),
            DeviceType::Display => Ok("Display"),
            DeviceType::Peripheral(id) => {
                hal_interface::interface_name(*id).map_err(KernelError::HalError)
            }
        }
    }
}

/// Represents the lock state for a device.
///
/// When `Locked`, the contained `u32` is the owner/caller id currently holding the lock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LockState {
    /// Device is locked by the contained owner/caller id.
    Locked(u32),
    /// Device is not locked.
    Unlocked,
}

impl LockState {
    /// Returns a static string representation of this lock state.
    ///
    /// # Returns
    /// - `"Locked"` if the state is [`LockState::Locked`]
    /// - `"Unlocked"` if the state is [`LockState::Unlocked`]
    pub fn as_str(&self) -> &'static str {
        match self {
            LockState::Locked(_) => "Locked",
            LockState::Unlocked => "Unlocked",
        }
    }

    /// Indicates whether this state is locked.
    ///
    /// # Returns
    /// - `true` if [`LockState::Locked`]
    /// - `false` if [`LockState::Unlocked`]
    pub fn is_locked(&self) -> bool {
        match self {
            LockState::Locked(_) => true,
            LockState::Unlocked => false,
        }
    }
}

/// Manages lock state for built-in devices and delegates peripheral lock state to the HAL.
///
/// Built-in devices:
/// - Terminal: stored in `terminal_state`
/// - Display: stored in `display_state`
///
/// Peripherals (`DeviceType::Peripheral`) are managed by the HAL through [`Kernel::hal()`].
pub struct DevicesManager {
    terminal_state: LockState,
    display_state: LockState,
}

impl DevicesManager {
    /// Creates a new [`DevicesManager`] with all built-in devices unlocked.
    ///
    /// # Returns
    /// - A new [`DevicesManager`] instance.
    pub fn new() -> Self {
        DevicesManager {
            terminal_state: LockState::Unlocked,
            display_state: LockState::Unlocked,
        }
    }

    /// Checks whether the given device is currently locked.
    ///
    /// # Parameters
    /// - `device_type`: The device to query.
    ///
    /// # Returns
    /// - `Ok(true)` if the device is locked.
    /// - `Ok(false)` if the device is unlocked.
    ///
    /// # Errors
    /// - For [`DeviceType::Peripheral`], returns `Err(KernelError::HalError(_))` if the HAL query
    ///   fails.
    pub fn is_locked(&self, device_type: DeviceType) -> KernelResult<bool> {
        match device_type {
            DeviceType::Terminal => Ok(self.terminal_state.is_locked()),
            DeviceType::Display => Ok(self.display_state.is_locked()),
            DeviceType::Peripheral(id) => Ok(Kernel::hal()
                .is_interface_locked(id)
                .map_err(KernelError::HalError)?
                .is_some()),
        }
    }

    /// Locks the given device for `caller_id`.
    ///
    /// For terminal/display:
    /// - If the device is unlocked, it becomes locked by `caller_id`.
    /// - If the device is already locked by `caller_id`, this is a no-op (`Ok(())`).
    /// - If the device is locked by someone else:
    ///   - `caller_id == KERNEL_MASTER_ID` takes ownership (lock takeover) and returns `Ok(())`.
    ///   - otherwise returns [`KernelError::DeviceLocked`].
    ///
    /// For peripherals, the operation is delegated to the HAL.
    ///
    /// # Parameters
    /// - `device_type`: The device to lock.
    /// - `caller_id`: The id of the caller attempting to lock the device.
    ///
    /// # Returns
    /// - `Ok(())` if the lock was acquired or already held by `caller_id`.
    ///
    /// # Errors
    /// - `Err(KernelError::DeviceLocked(_))` if the device is locked by a different owner and the
    ///   caller is not [`KERNEL_MASTER_ID`]. The error message uses [`DeviceType::name`].
    /// - `Err(KernelError::HalError(_))` for HAL failures when locking peripherals or when resolving
    ///   a peripheral name for error reporting.
    pub fn lock(&mut self, device_type: DeviceType, caller_id: u32) -> KernelResult<()> {
        match device_type {
            DeviceType::Terminal => match self.terminal_state {
                LockState::Unlocked => {
                    self.terminal_state = LockState::Locked(caller_id);
                    Ok(())
                }
                LockState::Locked(id) => {
                    if caller_id == id {
                        Ok(())
                    } else if caller_id == KERNEL_MASTER_ID {
                        self.terminal_state = LockState::Locked(caller_id);
                        Ok(())
                    } else {
                        Err(KernelError::DeviceLocked(device_type.name()?))
                    }
                }
            },
            DeviceType::Display => match self.display_state {
                LockState::Unlocked => {
                    self.display_state = LockState::Locked(caller_id);
                    Ok(())
                }
                LockState::Locked(id) => {
                    if caller_id == id {
                        Ok(())
                    } else if caller_id == KERNEL_MASTER_ID {
                        self.display_state = LockState::Locked(caller_id);
                        Ok(())
                    } else {
                        Err(KernelError::DeviceLocked(device_type.name()?))
                    }
                }
            },
            DeviceType::Peripheral(id) => Kernel::hal()
                .lock_interface(id, caller_id)
                .map_err(KernelError::HalError),
        }
    }

    /// Unlocks the given device if `caller_id` is authorized to do so.
    ///
    /// For terminal/display:
    /// - If the device is locked by `caller_id` or `caller_id == KERNEL_MASTER_ID`, it is unlocked.
    /// - If the device is locked by someone else, returns [`KernelError::DeviceNotOwned`].
    /// - If already unlocked, this is a no-op (`Ok(())`).
    ///
    /// For peripherals, the operation is delegated to the HAL.
    ///
    /// # Parameters
    /// - `device_type`: The device to unlock.
    /// - `caller_id`: The id of the caller attempting to unlock the device.
    ///
    /// # Returns
    /// - `Ok(())` if the device was unlocked or was already unlocked.
    ///
    /// # Errors
    /// - `Err(KernelError::DeviceNotOwned(_))` if the device is locked by a different owner and the
    ///   caller is not [`KERNEL_MASTER_ID`]. The error message uses [`DeviceType::name`].
    /// - `Err(KernelError::HalError(_))` for HAL failures when unlocking peripherals or when
    ///   resolving a peripheral name for error reporting.
    pub fn unlock(&mut self, device_type: DeviceType, caller_id: u32) -> KernelResult<()> {
        match device_type {
            DeviceType::Terminal => match self.terminal_state {
                LockState::Locked(id) => {
                    if caller_id == id || caller_id == KERNEL_MASTER_ID {
                        self.terminal_state = LockState::Unlocked;
                        Ok(())
                    } else {
                        Err(KernelError::DeviceNotOwned(device_type.name()?))
                    }
                }
                LockState::Unlocked => Ok(()),
            },
            DeviceType::Display => match self.display_state {
                LockState::Locked(id) => {
                    if caller_id == id || caller_id == KERNEL_MASTER_ID {
                        self.display_state = LockState::Unlocked;
                        Ok(())
                    } else {
                        Err(KernelError::DeviceNotOwned(device_type.name()?))
                    }
                }
                LockState::Unlocked => Ok(()),
            },
            DeviceType::Peripheral(id) => Kernel::hal()
                .unlock_interface(id, caller_id)
                .map_err(KernelError::HalError),
        }
    }

    /// Authorizes an action against the given device for `caller_id` without changing lock state.
    ///
    /// For terminal/display:
    /// - If locked: authorization succeeds only if `caller_id` is the owner or is
    ///   [`KERNEL_MASTER_ID`].
    /// - If unlocked: always succeeds.
    ///
    /// For peripherals, authorization is delegated to the HAL.
    ///
    /// # Parameters
    /// - `device_type`: The device to authorize access for.
    /// - `caller_id`: The id of the caller requesting authorization.
    ///
    /// # Returns
    /// - `Ok(())` if the caller is authorized to act on the device.
    ///
    /// # Errors
    /// - `Err(KernelError::DeviceNotOwned(_))` if the device is locked by a different owner and the
    ///   caller is not [`KERNEL_MASTER_ID`]. The error message uses [`DeviceType::name`].
    /// - `Err(KernelError::HalError(_))` for HAL failures when authorizing peripherals or when
    ///   resolving a peripheral name for error reporting.
    pub fn authorize(&mut self, device_type: DeviceType, caller_id: u32) -> KernelResult<()> {
        match device_type {
            DeviceType::Terminal => match self.terminal_state {
                LockState::Locked(id) => {
                    if caller_id == id || caller_id == KERNEL_MASTER_ID {
                        Ok(())
                    } else {
                        Err(KernelError::DeviceNotOwned(device_type.name()?))
                    }
                }
                LockState::Unlocked => Ok(()),
            },
            DeviceType::Display => match self.display_state {
                LockState::Locked(id) => {
                    if caller_id == id || caller_id == KERNEL_MASTER_ID {
                        Ok(())
                    } else {
                        Err(KernelError::DeviceNotOwned(device_type.name()?))
                    }
                }
                LockState::Unlocked => Ok(()),
            },
            DeviceType::Peripheral(id) => Kernel::hal()
                .authorize_action(id, caller_id)
                .map_err(KernelError::HalError),
        }
    }
}
