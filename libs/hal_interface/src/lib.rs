#![no_std]

mod bindings;
mod errors;
mod interface_actions;

pub use interface_actions::*;

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

    /// Executes a specified interface action on a given hardware interface, identified by its ID,
    /// and translates the action result into a `HalResult`.
    ///
    /// # Parameters
    /// - `id`: The identifier (index) of the hardware interface where the action needs to be performed.
    /// - `action`: The `InterfaceActions` enum instance that specifies the action to execute.
    ///   The actions currently supported are:
    ///   - `InterfaceActions::GpioWrite`: Performs a GPIO write operation.
    ///   - `InterfaceActions::UartWrite`: Performs a UART write operation.
    ///
    /// # Returns
    /// - `HalResult<()>`: A result indicating the outcome of the action.
    ///   - On success: Returns an empty `Ok(())`, implying the action was executed successfully.
    ///   - On failure: Returns an error describing the issue that occurred.
    ///
    /// # Safety
    /// - The `gpio
    pub fn interface_action(&mut self, id: usize, action: InterfaceActions) -> HalResult<()> {
        match action {
            InterfaceActions::GpioWrite(act) => unsafe {
                gpio_write(id as u8, act).to_result(Some(id), None, Some(action))
            },
            InterfaceActions::UartWrite(act) => {
                act.action(id as u8).to_result(Some(id), None, Some(action))
            }
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
