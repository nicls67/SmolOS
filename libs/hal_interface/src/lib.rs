#![no_std]

mod bindings;
mod errors;
mod interface_read;
mod interface_write;
mod lock;

pub use interface_read::*;
pub use interface_write::*;

use crate::bindings::{HalInterfaceResult, get_core_clk, get_interface_id, gpio_write, hal_init};
use crate::lock::Locker;
pub use errors::*;

pub struct Hal {
    locker: Locker,
}

impl Hal {
    /// Initializes a new instance of the struct.
    ///
    /// # Parameters
    /// - `master_id`: A `u32` representing the ID of the master resource or entity to be associated with the instance.
    ///
    /// # Returns
    /// An instance of `Self` with the specified `master_id`.
    ///
    /// # Safety
    /// This function calls the `hal_init()` function, which is marked as `unsafe`. Ensure that `hal_init()` is safe to call
    /// in the context where this function is used, as it may involve accessing or modifying low-level hardware or memory.
    ///
    pub fn new(master_id: u32) -> Self {
        unsafe { hal_init() }
        Self {
            locker: Locker::new(master_id),
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
                self.locker.add_interface(id as usize);
                Ok(id as usize)
            }
            HalInterfaceResult::ErrInterfaceNotFound => Err(HalError::InterfaceNotFound(name)),
            _ => Err(HalError::UnknownError),
        }
    }

    /// Locks a specific interface using a given locker ID.
    ///
    /// This method attempts to lock an interface identified by its `id` using a provided `locker_id`.
    /// It delegates the locking operation to the `locker` instance within the current object.
    ///
    /// # Parameters
    ///
    /// - `id`: The unique identifier of the interface to be locked.
    /// - `locker_id`: The identifier of the entity attempting to lock the interface.
    ///
    /// # Returns
    ///
    /// - `HalResult<()>`: Returns `Ok(())` if the operation is successful or an appropriate error
    ///   wrapped in `HalResult` if the operation fails.
    ///
    /// # Errors
    ///
    /// This method will return an error in the following cases:
    /// - The interface identified by `id` is already locked.
    /// - The `locker_id` is invalid or does not have the required permissions.
    ///
    pub fn lock_interface(&mut self, id: usize, locker_id: u32) -> HalResult<()> {
        self.locker.lock_interface(id, locker_id)
    }

    /// Unlocks a specific interface identified by its ID.
    ///
    /// This function delegates the unlocking operation to the underlying locker
    /// system by passing the interface ID and the locker ID. It is used to re-enable
    /// access to an interface that has previously been locked.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the interface to be unlocked.
    /// * `locker_id` - The identifier of the locking entity or system that is
    ///   performing the unlock operation.
    ///
    /// # Returns
    ///
    /// * `HalResult<()>` - Returns a result indicating the success (`Ok`) or error
    ///   (`Err`) of the unlocking operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying locker system encounters
    /// an issue while unlocking the specified interface.
    ///
    pub fn unlock_interface(&mut self, id: usize, locker_id: u32) -> HalResult<()> {
        self.locker.unlock_interface(id, locker_id)
    }

    /// Authorizes an action for a given locker and user.
    ///
    /// This method delegates the authorization process to the `authorize_action`
    /// method of the `locker` instance and returns the result of that operation.
    ///
    /// # Parameters
    /// - `id`: The unique identifier of the user for whom the action is being authorized.
    /// - `locker_id`: The unique identifier of the locker for which the action is being authorized.
    ///
    /// # Returns
    /// - `HalResult<()>`: Returns a result indicating success or failure of the authorization process.
    ///
    /// If the authorization process completes successfully, the result will be `Ok(())`. If there is
    /// an error during authorization, the result will be an `Err` containing the error details.
    ///
    /// # Errors
    /// This method may return an error if:
    /// - The specified user ID is invalid or does not exist.
    /// - The specified locker ID is invalid or does not exist.
    /// - The authorization fails due to business logic constraints.
    ///
    pub fn authorize_action(&mut self, id: usize, locker_id: u32) -> HalResult<()> {
        self.locker.authorize_action(id, locker_id)
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
        self.locker.authorize_action(ressource_id, caller_id)?;

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

    pub fn interface_read(
        &mut self,
        ressource_id: usize,
        caller_id: u32,
        read_action: InterfaceReadAction,
    ) -> HalResult<InterfaceReadResult> {
        // Check for lock on interface
        self.locker.authorize_action(ressource_id, caller_id)?;

        // Perform action
        let read_result;
        let interface_res;

        match read_action {
            InterfaceReadAction::LcdRead(act) => {
                let mut lcd_result = LcdRead::LcdSize(0, 0);
                interface_res = act.read(ressource_id, &mut lcd_result);
                read_result = InterfaceReadResult::LcdRead(lcd_result);
            }
        };
        match interface_res.to_result(Some(ressource_id), None, None, Some(read_action)) {
            Ok(_) => Ok(read_result),
            Err(e) => Err(e),
        }
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
