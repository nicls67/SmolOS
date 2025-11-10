#![no_std]

mod bindings;
mod errors;
mod interface_read;
mod interface_write;
mod lock;

use heapless::Vec;
pub use interface_read::*;
pub use interface_write::*;

use crate::bindings::{
    HalInterfaceResult, configure_callback, get_core_clk, get_interface_id, get_read_buffer,
    gpio_write, hal_init,
};
use crate::lock::Locker;
pub use errors::*;

pub const BUFFER_SIZE: usize = 32;

pub struct Hal {
    locker: Option<Locker>,
}

pub type InterfaceCallback = extern "C" fn(u8);

impl Hal {
    /// Creates a new instance of the struct.
    ///
    /// This function initializes the hardware abstraction layer by
    /// calling an unsafe `hal_init` function and then constructs
    /// the instance of the struct with default values.
    ///
    /// # Safety
    /// The internal call to `hal_init` is marked as `unsafe`; ensure that
    /// calling this function aligns with the requirements of that unsafe
    /// function.
    ///
    /// # Returns
    /// A new instance of the struct with the `locker` property set to `None`.
    ///
    pub fn new() -> Self {
        unsafe { hal_init() }
        Self { locker: None }
    }

    /// Configures the locker with a master lock ID if it has not been previously configured.
    ///
    /// # Parameters
    /// - `master_lock_id` (u32): An identifier for the master lock to configure the locker.
    ///
    /// # Returns
    /// - `HalResult<()>`:
    ///   - `Ok(())` if the locker is successfully configured.
    ///   - `Err(HalError::LockerAlreadyConfigured)` if the locker has already been configured.
    ///
    /// # Errors
    /// This function returns an error if the locker is already configured to prevent reconfiguration.
    ///
    pub fn configure_locker(&mut self, master_lock_id: u32) -> HalResult<()> {
        if self.locker.is_none() {
            self.locker = Some(Locker::new(master_lock_id));
            Ok(())
        } else {
            Err(HalError::LockerAlreadyConfigured)
        }
    }

    /// Fetches the unique identifier (ID) of a hardware interface by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - A static string slice representing the name of the hardware interface.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - The interface ID as a `usize` if the interface is successfully found.
    /// * `Err(HalError)` - Returns an appropriate error if the interface could not be found or another issue occurs:
    ///     - `HalError::InterfaceNotFound(name)` - Returned if the specified interface is not found.
    ///     - `HalError::UnknownError` - Returned in case of an unknown error.
    ///
    /// # Behavior
    ///
    /// * This function internally calls an external `unsafe` function `get_interface_id`.
    ///   - If the result is `HalInterfaceResult::OK`, it registers the interface ID
    ///     in the `locker` and returns the ID.
    ///   - If the interface is not found, it returns the `HalError::InterfaceNotFound` error.
    ///   - For any other result, it returns a generic `HalError::UnknownError`.
    ///
    /// # Safety
    ///
    /// * The usage of `unsafe` code is required due to calling and interacting with the
    ///   external `get_interface_id` function. As such, it is assumed that `get_interface_id`
    ///   operates correctly and adheres to its expected behavior.
    ///
    /// # Errors
    ///
    /// This function may fail if:
    /// * The requested interface does not exist.
    /// * An unknown error occurs during the ID lookup.
    pub fn get_interface_id(&mut self, name: &'static str) -> HalResult<usize> {
        let mut id = 0;
        match unsafe { get_interface_id(name.as_ptr(), &mut id) } {
            HalInterfaceResult::OK => {
                if let Some(locker) = &mut self.locker {
                    locker.add_interface(id as usize);
                }
                Ok(id as usize)
            }
            HalInterfaceResult::ErrInterfaceNotFound => Err(HalError::InterfaceNotFound(name)),
            _ => Err(HalError::UnknownError),
        }
    }

    /// Locks a specific interface using the provided locker identifier.
    ///
    /// This function attempts to lock an interface with the given `id` by delegating
    /// the operation to an internal `locker` if available. The locking mechanism
    /// ensures that only the specified `locker_id` has exclusive access to the interface.
    ///
    /// # Parameters
    /// - `id`: The unique identifier of the interface to be locked.
    /// - `locker_id`: The identifier of the locker requesting access.
    ///
    /// # Returns
    /// - `HalResult<()>`: On success, returns `Ok(())`. If locking fails, it propagates
    ///   an error from the `locker`.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The underlying `locker` encounters an issue while locking the interface.
    ///
    /// # Notes
    /// - If the internal `locker` is not initialized (`None`), this function will simply
    ///   return `Ok(())` without performing any lock operation.
    pub fn lock_interface(&mut self, id: usize, locker_id: u32) -> HalResult<()> {
        if let Some(locker) = &mut self.locker {
            locker.lock_interface(id, locker_id)?;
        }
        Ok(())
    }

    /// Unlocks a specific interface by its ID using the provided locker ID.
    ///
    /// This function attempts to unlock an interface identified by the `id` parameter.
    /// It utilizes the provided `locker_id` to perform the unlock operation. If the locker is present,
    /// the method delegates the unlock functionality to the `locker.unlock_interface` method. Any errors
    /// encountered during this process will be propagated as a `HalResult` error.
    ///
    /// # Parameters
    /// - `id`: The unique identifier of the interface to be unlocked.
    /// - `locker_id`: The identifier of the locker used to authorize the unlocking process.
    ///
    /// # Returns
    /// - `HalResult<()>`: Returns `Ok(())` if the interface was successfully unlocked or if no locker exists.
    ///   Propagates any error returned by the `locker.unlock_interface` method.
    ///
    /// # Errors
    /// - This function returns a propagated error from the `locker.unlock_interface` method if the unlocking
    ///   process fails.
    ///
    pub fn unlock_interface(&mut self, id: usize, locker_id: u32) -> HalResult<()> {
        if let Some(locker) = &mut self.locker {
            locker.unlock_interface(id, locker_id)?;
        }
        Ok(())
    }

    /// Authorizes an action for a given entity based on its ID and associated locker ID.
    ///
    /// This function attempts to authorize an action by delegating the authorization
    /// to an internal `locker` component if it exists. The provided `id` and `locker_id`
    /// are used to perform the authorization.
    ///
    /// # Parameters
    /// - `id`: A `usize` representing the identifier of the entity requesting the action.
    /// - `locker_id`: A `u32` representing the identifier of the associated locker.
    ///
    /// # Returns
    /// - `HalResult<()>`: Returns `Ok(())` if the authorization is successful or the `locker`
    ///   is not present. Returns an error wrapped in `HalResult` if the authorization process fails.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The `locker` is present but fails to authorize the action due to invalid input,
    ///   mismatched IDs, or other internal validation criteria.
    ///
    /// # Panics
    /// This method will not panic.
    ///
    /// # Note
    /// If the `locker` is `None`, this function will return `Ok(())` without performing
    /// any authorization.
    pub fn authorize_action(&mut self, id: usize, locker_id: u32) -> HalResult<()> {
        if let Some(locker) = &mut self.locker {
            locker.authorize_action(id, locker_id)?;
        }
        Ok(())
    }

    /// Performs a write operation on the specified interface based on the action provided.
    ///
    /// # Parameters
    /// - `id`: The unique identifier of the interface to act upon. This parameter is of type `usize`.
    /// - `action`: The specific action to be performed on the interface. This is an enum of type `InterfaceActions`,
    ///   which determines what kind of operation will be executed (e.g., `GpioWrite`, `UartWrite`, `Lcd`).
    ///
    /// # Returns
    /// - `HalResult<()>`: Result indicating the success or failure of the operation. If the operation executes successfully,
    ///   the result contains `()`. Errors are propagated through the `HalResult` type.
    ///
    /// # Behavior
    /// - The function matches on the provided `InterfaceActions`:
    ///   - `InterfaceActions::GpioWrite`: Calls the `gpio_write` function, passing the `id` (as `u8`) and the provided action.
    ///     Converts the result of `gpio_write` into a `HalResult` using `to_result()`.
    ///   - `InterfaceActions::UartWrite`: Executes the action linked to the `UartWrite` operation by calling its `action` method
    ///     with the `id` (as `u8`), then processes its result with `to_result()`.
    ///   - `InterfaceActions::Lcd`: Similar to `UartWrite`, it calls the `action` method for LCD, passing the `id`
    ///     (as `u8`) and processes its result using `to_result()`.
    ///
    /// # Safety
    /// - The `GpioWrite` case executes an `unsafe` block when invoking the `gpio_write` function. Ensure that the usage
    ///   of the unsafe code does not introduce undefined behavior.
    ///
    /// # Conversion
    /// - The `to_result` method is used in all cases to convert the invoked action's return value into an ` HalResult `
    ///   while providing context with the optional identifiers (`id`, `action`).
    ///
    pub fn interface_write(
        &mut self,
        ressource_id: usize,
        caller_id: u32,
        action: InterfaceWriteActions,
    ) -> HalResult<()> {
        // Check for lock on interface
        if let Some(locker) = &mut self.locker {
            locker.authorize_action(ressource_id, caller_id)?;
        }

        // Perform action
        match action {
            InterfaceWriteActions::GpioWrite(act) => unsafe {
                gpio_write(ressource_id as u8, act).to_result(
                    Some(ressource_id),
                    None,
                    Some(action),
                    None,
                )
            },
            InterfaceWriteActions::UartWrite(act) => act.action(ressource_id as u8).to_result(
                Some(ressource_id),
                None,
                Some(action),
                None,
            ),
            InterfaceWriteActions::Lcd(act) => act.action(ressource_id as u8).to_result(
                Some(ressource_id),
                None,
                Some(action),
                None,
            ),
        }
    }

    /// Reads from a specified interface resource using an authorized caller.
    ///
    /// # Parameters
    ///
    /// * `ressource_id` - The unique identifier of the resource to be read.
    /// * `caller_id` - The unique identifier of the caller requesting the read action.
    /// * `read_action` - The specific action to be performed, in this case, an `InterfaceReadAction`.
    ///
    /// # Returns
    ///
    /// If successful, returns a `HalResult` containing an `InterfaceReadResult` which encapsulates
    /// the result of the read action (e.g., LCD size data).
    ///
    /// # Errors
    ///
    /// This function may return an error in the following cases:
    /// * If authorization fails because the caller is not permitted access to the requested resource.
    /// * If the `read_action` fails to perform the read operation.
    /// * Any other issue encountered while processing the request is wrapped in the resulting error.
    ///
    /// # Workflow
    ///
    /// 1. Checks if there is a lock on the interface resource and, if so, authorizes the caller
    ///    by delegating to the resource's locker (if any).
    /// 2. Executes the provided `read_action` for the given resource ID. The specific implementation
    ///    details of the `read_action` are handled by the provided `InterfaceReadAction` implementation.
    /// 3. Converts the result of the `read_action` into an `InterfaceReadResult` and associates
    ///    it with the calling context for error handling consistency.
    /// 4. Returns an `Ok` or an `Err` based on the result of the operation.
    ///
    /// # Notes
    ///
    /// * The function assumes that the `InterfaceReadAction` is properly implemented to handle
    ///   the reading operation and return the expected data.
    /// * Any locking or resource management is delegated to the `locker`'s `authorize_action` method.
    pub fn interface_read(
        &mut self,
        ressource_id: usize,
        caller_id: u32,
        read_action: InterfaceReadAction,
    ) -> HalResult<InterfaceReadResult> {
        // Check for lock on interface
        if let Some(locker) = &mut self.locker {
            locker.authorize_action(ressource_id, caller_id)?;
        }

        // Perform action
        let read_result;
        let interface_res;

        match read_action {
            InterfaceReadAction::LcdRead(act) => {
                let mut lcd_result = LcdRead::LcdSize(0, 0);
                interface_res = act.read(ressource_id, &mut lcd_result);
                read_result = InterfaceReadResult::LcdRead(lcd_result);
            }
            InterfaceReadAction::BufferRead => {
                // Initialize the buffer pointer
                let mut buffer: &mut RxBuffer = &mut RxBuffer {
                    buffer: core::ptr::null_mut(),
                    size: 0,
                };

                // Get buffer address
                unsafe {
                    interface_res = get_read_buffer(ressource_id as u8, &mut buffer);
                }
                // Copy buffer content into Vec
                let mut vec: Vec<u8, BUFFER_SIZE> = Vec::new();
                for i in 0..buffer.size {
                    unsafe {
                        vec.push(*buffer.buffer.wrapping_add(i as usize)).unwrap();
                    }
                }
                read_result = InterfaceReadResult::BufferRead(vec);
                // Re-initialize buffer
                buffer.size = 0;
            }
        };
        match interface_res.to_result(Some(ressource_id), None, None, Some(read_action)) {
            Ok(_) => Ok(read_result),
            Err(e) => Err(e),
        }
    }

    /// Configures a callback interface with the given parameters.
    ///
    /// # Parameters
    /// - `ressource_id`: An identifier for the resource (of type `usize`) that the callback is associated with.
    /// - `caller_id`: The unique identifier (of type `u32`) for the caller or entity requesting the configuration.
    /// - `callback`: The callback function or interface (of type `InterfaceCallback`) to be associated with the resource.
    ///
    /// # Returns
    /// - `HalResult<()>`: Returns a `HalResult` indicating success (`Ok`) or an error (`Err`) in case of failure during the configuration process.
    ///
    /// # Behavior
    /// 1. Ensures that the caller is authorized to perform the action using the `locker` mechanism, if it is present.
    ///    - If the `self.locker` field is set and contains a locker, the `authorize_action` method is invoked with the provided `ressource_id` and `caller_id`.
    ///    - If authorization fails, it propagates the error returned by `authorize_action`.
    /// 2. Configures the callback by calling the `configure_callback` method in an unsafe block.
    ///    - Converts the `ressource_id` from `usize` to `u8` as required by the low-level `configure_callback` implementation.
    ///    - Wraps the result of `configure_callback` in a `HalResult` using the `to_result` method, with `ressource_id` as additional context in case of associated errors.
    ///
    /// # Safety
    /// - The function contains an `unsafe` block while invoking the external `configure_callback` function. The caller must ensure that:
    ///   - The provided `ressource_id` and `callback` adhere to expected invariants and constraints.
    ///   - The conversion of `ressource_id` to a smaller type (`u8`) does not lead to truncation or incorrect resource mapping.
    ///
    /// # Errors
    /// - Returns an error in the following situations:
    ///   - If the authorization check via the `locker.authorize_action` method fails.
    ///   - If the underlying `configure_callback` invocation fails due to invalid parameters or other reasons.
    ///
    pub fn configure_callback(
        &mut self,
        ressource_id: usize,
        caller_id: u32,
        callback: InterfaceCallback,
    ) -> HalResult<()> {
        // Check for lock on interface
        if let Some(locker) = &mut self.locker {
            locker.authorize_action(ressource_id, caller_id)?;
        }

        // Configure callback
        unsafe { configure_callback(ressource_id as u8, callback) }.to_result(
            Some(ressource_id),
            None,
            None,
            None,
        )
    }

    /// Retrieves the current core clock frequency.
    ///
    /// # Returns
    ///
    /// Returns an unsigned 32-bit integer representing the core clock frequency
    /// in hertz (Hz).
    ///
    /// # Safety
    ///
    /// This function internally calls an unsafe function `get_core_clk()`.
    /// The unsafe block assumes that `get_core_clk()` is implemented correctly
    /// and adheres to any safety guarantees defined for it.
    ///
    pub fn get_core_clk(&self) -> u32 {
        unsafe { get_core_clk() }
    }
}
