#![no_std]

mod bindings;
mod errors;
mod interface_read;
mod interface_write;

pub use interface_read::*;
pub use interface_write::*;

use crate::bindings::{HalInterfaceResult, get_core_clk, get_interface_id, gpio_write, hal_init};
pub use errors::*;

pub struct Hal;

impl Default for Hal {
    fn default() -> Self {
        Self::new()
    }
}

impl Hal {
    /// Creates a new instance of the struct.
    ///
    /// This function initializes the hardware abstraction layer (HAL) by calling
    /// the `hal_init()` function. The `hal_init()` function is marked as `unsafe`,
    /// meaning it could perform operations that may break memory safety or depend on specific
    /// hardware context.
    ///
    /// # Safety
    /// - Ensure that all preconditions for calling `hal_init()` are met.
    /// - This function directly utilizes unsafe code and should therefore be used with caution.
    ///
    /// # Returns
    /// - A new instance of the struct.
    ///
    pub fn new() -> Self {
        unsafe { hal_init() }
        Self
    }

    /// Retrieves the interface ID associated with a given interface name.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice (`'static`) representing the name of the interface.
    ///
    /// # Returns
    ///
    /// If the provided `name` corresponds to a valid interface, the function will return
    /// a `HalResult<usize>` containing the ID of the interface. The ID is returned as a `usize`.
    ///
    /// On failure, the function will return a `HalError::InterfaceNotFound`
    /// error encapsulated in the `HalResult::Err` variant. This occurs
    /// if the interface name is not found.
    ///
    /// # Safety
    ///
    /// This function calls an unsafe block relying on `get_interface_id`,
    /// which assumes that the provided `name` string pointer and mutable ID
    /// reference (`&mut id`) are valid. Ensure that the memory integrity
    /// and the lifecycle of the provided variables are guaranteed when using
    /// this function.
    ///
    /// # Errors
    ///
    /// * Returns `Err(HalError::InterfaceNotFound)` if the interface name
    ///   does not exist.
    ///
    /// # Notes
    ///
    /// This function is part of a HAL (Hardware Abstraction Layer) implementation
    /// and assumes that the HAL interface provides an external `get_interface_id`
    /// function with appropriate error handling via the `HalInterfaceResult` enumeration.
    pub fn get_interface_id(&self, name: &'static str) -> HalResult<usize> {
        let mut id = 0;
        match unsafe { get_interface_id(name.as_ptr(), &mut id) } {
            HalInterfaceResult::OK => Ok(id as usize),
            HalInterfaceResult::ErrInterfaceNotFound => Err(HalError::InterfaceNotFound(name)),
            _ => Err(HalError::UnknownError),
        }
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
    pub fn interface_write(&mut self, id: usize, action: InterfaceWriteActions) -> HalResult<()> {
        match action {
            InterfaceWriteActions::GpioWrite(act) => unsafe {
                gpio_write(id as u8, act).to_result(Some(id), None, Some(action), None)
            },
            InterfaceWriteActions::UartWrite(act) => {
                act.action(id as u8)
                    .to_result(Some(id), None, Some(action), None)
            }
            InterfaceWriteActions::Lcd(act) => {
                act.action(id as u8)
                    .to_result(Some(id), None, Some(action), None)
            }
        }
    }

    pub fn interface_read(
        &mut self,
        id: usize,
        read_action: InterfaceReadAction,
    ) -> HalResult<InterfaceReadResult> {
        let read_result;
        let interface_res;

        match read_action {
            InterfaceReadAction::LcdRead(act) => {
                let mut lcd_result = LcdRead::LcdSize(0, 0);
                interface_res = act.read(id, &mut lcd_result);
                read_result = InterfaceReadResult::LcdRead(lcd_result);
            }
        };
        match interface_res.to_result(Some(id), None, None, Some(read_action)) {
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
