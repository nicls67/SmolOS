use crate::HalError::{
    IncompatibleAction, InterfaceInitError, ReadOnlyInterface, WriteOnlyInterface, WrongInterfaceId,
};
use crate::HalErrorLevel::{Critical, Error, Fatal};
use crate::{HalResult, InterfaceReadActions, InterfaceWriteActions};
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::Uart;
use heapless::Vec;

pub enum InterfaceType {
    GpioOutput(Output<'static>),
    Uart(Uart<'static, Async>),
}

pub struct Interface {
    pub(crate) name: &'static str,
    pub(crate) interface: InterfaceType,
}

impl Interface {
    pub fn new(name: &'static str, interface: InterfaceType) -> Interface {
        Interface { name, interface }
    }
}

/// A struct representing a fixed-capacity vector containing elements of type `Interface`.
///
/// This structure wraps around a `Vec` with a capacity limited to 256 elements of type `Interface`.
///
/// # Fields
/// - `vect`: A vector holding the elements of type `Interface` with a maximum capacity of 256.
///
/// Note: Ensure that the type `Interface` is properly defined elsewhere in your codebase.
pub struct InterfaceVect {
    vect: Vec<Interface, 256>,
}

impl InterfaceVect {
    /// Creates a new instance of `InterfaceVect`.
    ///
    /// This method initializes an `InterfaceVect` with an empty vector.
    ///
    /// # Returns
    /// A new `InterfaceVect` instance containing an empty `Vec`.
    ///
    pub fn new() -> InterfaceVect {
        InterfaceVect { vect: Vec::new() }
    }

    /// Adds a new interface to the list of interfaces managed by the HAL (Hardware Abstraction Layer).
    ///
    /// # Arguments
    ///
    /// * `interface` - The `Interface` object that will be added to the internal list of interfaces.
    ///
    /// # Returns
    ///
    /// * `HalResult<()>` - Returns `Ok(())` if the interface is successfully added to the list.
    ///   Returns an error of type `InterfaceInitError` if the interface could not be added
    ///   (e.g., the interfaces list is full).
    ///
    /// # Errors
    ///
    /// This function will return an `InterfaceInitError` with a custom message if the internal
    /// `vect` list reaches its capacity, and the new interface cannot be added.
    ///
    pub fn add_interface(&mut self, interface: Interface) -> HalResult<()> {
        self.vect
            .push(interface)
            .map_err(|_| InterfaceInitError(Fatal, "interfaces list is full"))?;

        Ok(())
    }

    /// Retrieves the unique ID (index) of an interface by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - A static string slice representing the name of the interface to search for.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - The index of the interface in the internal vector if found.
    /// * `Err(InterfaceInitError)` - An error indicating that the interface with the specified
    ///   name could not be found. The error includes the name and a critical error level.
    ///
    /// # Errors
    ///
    /// This function returns `InterfaceInitError` if the provided interface name does not
    /// exist in the internal storage.
    ///
    /// # Notes
    ///
    /// This function performs a linear search on the `vect` field, which may impact performance
    /// for a large number of stored interfaces.
    pub fn get_interface_id(&self, name: &'static str) -> HalResult<usize> {
        for (i, ift) in self.vect.iter().enumerate() {
            if ift.name == name {
                return Ok(i);
            }
        }
        Err(InterfaceInitError(Critical, name))
    }

    pub fn interface_write(&mut self, id: usize, action: InterfaceWriteActions) -> HalResult<()> {
        let interface = self
            .vect
            .get_mut(id)
            .ok_or(WrongInterfaceId(Critical, id))?;

        match &mut interface.interface {
            InterfaceType::GpioOutput(pin) => {
                if let InterfaceWriteActions::GpioWrite(action) = action {
                    action.action(pin);
                } else {
                    return Err(IncompatibleAction(Error, action.name(), interface.name));
                }
            }
            InterfaceType::Uart(uart) => {
                if let InterfaceWriteActions::UartWrite(action) = action {
                    action.action(uart);
                } else {
                    return Err(IncompatibleAction(Error, action.name(), interface.name));
                }
            }
            _ => return Err(ReadOnlyInterface(Error, interface.name)),
        }

        Ok(())
    }

    pub fn interface_read(&mut self, id: usize, action: InterfaceReadActions) -> HalResult<()> {
        let interface = self
            .vect
            .get_mut(id)
            .ok_or(WrongInterfaceId(Critical, id))?;

        match &mut interface.interface {
            InterfaceType::Uart(uart) => {
                if let InterfaceReadActions::UartRead(mut action) = action {
                    action.action(uart);
                } else {
                    return Err(IncompatibleAction(Error, action.name(), interface.name));
                }
            }
            _ => return Err(WriteOnlyInterface(Error, interface.name)),
        }

        Ok(())
    }
}
