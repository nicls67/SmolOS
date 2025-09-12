//! This module defines the `HalError` and `HalErrorLevel` enumerations and their associated
//! functionality. It provides a structured way to represent hardware abstraction layer (HAL)
//! related errors with different severity levels and format
use crate::HalError::{
    HalAlreadyLocked, HalNotLocked, IncompatibleAction, InterfaceInitError, InterfaceNotFound,
    ReadError, ReadOnlyInterface, WriteError, WriteOnlyInterface, WrongInterfaceId,
};
use crate::HalErrorLevel::{Critical, Error, Fatal};
use heapless::{String, format};

pub type HalResult<T> = Result<T, HalError>;

/// Represents the severity levels of hardware abstraction layer (HAL) errors.
///
/// This enum is used to categorize different error levels that might occur
/// within the HAL, allowing for appropriate handling based on severity.
///
/// # Variants
///
/// - `Fatal`
///   Indicates a critical error that prevents the system from functioning
///   and requires an immediate shutdown or restart.
///
/// - `Critical`
///   Represents a high-severity error that could cause significant issues or
///   instability in the system, but may still allow for limited operation.
///
/// - `Error`
///   Denotes a standard error that indicates a problem or failure in the HAL,
///   but is less severe than `Critical` or `Fatal` and might be recoverable.
///
#[derive(Debug, Clone, Copy)]
pub enum HalErrorLevel {
    Fatal,
    Critical,
    Error,
}

impl HalErrorLevel {
    /// Converts the `HalErrorLevel` enum variant into a corresponding string slice representation.
    ///
    /// # Returns
    ///
    /// - `"Fatal error : "` for `HalErrorLevel::Fatal` variant.
    /// - `"Critical error : "` for `HalErrorLevel::Critical` variant.
    /// - `"Error : "` for `HalErrorLevel::Error` variant.
    ///
    /// # Note
    ///
    /// This method provides a textual description of each `HalErrorLevel` variant, which can
    /// be used for logging or display purposes.
    pub fn as_str(&self) -> &str {
        match self {
            HalErrorLevel::Fatal => "HAL Fatal error : ",
            HalErrorLevel::Critical => "HAL Critical error : ",
            HalErrorLevel::Error => "HAL Error : ",
        }
    }
}

#[derive(Debug)]
pub enum HalError {
    HalAlreadyLocked,
    HalNotLocked,
    InterfaceInitError(&'static str),
    InterfaceNotFound(&'static str),
    WrongInterfaceId(usize),
    ReadOnlyInterface(&'static str),
    WriteOnlyInterface(&'static str),
    IncompatibleAction(&'static str, &'static str),
    WriteError(HalErrorLevel, &'static str),
    ReadError(HalErrorLevel, &'static str),
}

impl HalError {
    /// Converts the current object into a formatted string representation with a maximum size of 256 characters.
    ///
    /// This method generates a descriptive string representation for various error scenarios or states that
    /// the object can represent. Based on the variant of the enum or object in question, it produces a specific
    /// string with severity information and additional contextual details.
    ///
    /// # Returns
    /// A `String` with the descriptive message for the current object state or error.
    ///
    /// # Variants Handling
    /// - **HalAlreadyLocked**: Produces a message indicating that the hardware abstraction layer (HAL) is already locked.
    /// - **HalNotLocked**: Produces a message indicating that the HAL is not locked.
    /// - **InterfaceInitError(err)**: Produces a message describing an initialization error with an interface,
    ///   incorporating the specific error `err`.
    /// - **InterfaceNotFound(name)**: Produces a message indicating that an interface with the given `name` is not found.
    /// - **WrongInterfaceId(id)**: Produces a message indicating that the provided interface ID `id` does not exist.
    /// - **ReadOnlyInterface(name)**: Produces a message indicating that the interface named `name` is read-only.
    /// - **WriteOnlyInterface(name)**: Produces a message indicating that the interface named `name` is write-only.
    /// - **IncompatibleAction(action, interface)**: Produces a message indicating that the `action` is incompatible
    ///   with the given `interface`.
    /// - **WriteError(lvl, ift)**: Produces a message indicating an error during a write operation on the specified
    ///   interface `ift`.
    /// - **ReadError(lvl, ift)**: Produces a message indicating an error during a read operation on the specified
    ///   interface `ift`.
    ///
    /// # Behavior
    /// - Uses `self.severity().as_str()` to prefix the severity level to the message.
    /// - Handles string formatting carefully, limiting the size of the formatted content where applicable.
    /// - Avoids panics by using `.unwrap_or(())` to handle potential `push_str` failures in constrained environments.
    ///
    /// # Errors
    /// While the function itself does not return Result, it gracefully handles potential string exceeding capacity
    /// errors by using `unwrap_or(())` during the string construction process.
    ///
    /// # Note
    /// - The method assumes that `self.severity().as_str()` correctly maps the severity levels to a string representation.
    /// - Enforces a string capacity limit of 256 characters for safety.
    pub fn to_string(&self) -> String<256> {
        let mut msg = String::new();
        match self {
            HalAlreadyLocked => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str("HAL already locked").unwrap();
            }
            HalNotLocked => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str("HAL not locked").unwrap();
            }
            InterfaceInitError(err) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str("Interface initialization error : ").unwrap();
                msg.push_str(err).unwrap_or(());
            }
            InterfaceNotFound(name) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(30; "Interface {} not found", name)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            WrongInterfaceId(id) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(30; "Interface ID {} does not exist", id)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            ReadOnlyInterface(name) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(30; "Interface {} is read-only", name)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            WriteOnlyInterface(name) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(30; "Interface {} is write-only", name)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            IncompatibleAction(action, interface) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(70; "Action {} is not compatible with interface {}", action, interface)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            WriteError(lvl, ift) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(256; "Error during write on interface {} ", ift)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            ReadError(lvl, ift) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(256; "Error during read on interface {}", ift)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
        }
        msg
    }

    /// Returns the severity level of the `HalError` instance.
    ///
    /// This method analyzes the type of the `HalError` and maps it to a corresponding
    /// `HalErrorLevel`, which represents how critical the error is. The mapping for
    /// each specific error variant to its respective severity level is defined as follows:
    ///
    /// - `HalAlreadyLocked`: Returns `Error`
    /// - `HalNotLocked`: Returns `Error`
    /// - `InterfaceInitError(_)`: Returns `Fatal`
    /// - `InterfaceNotFound(_)`: Returns `Critical`
    /// - `WrongInterfaceId(_)`: Returns `Critical`
    /// - `ReadOnlyInterface(_)`: Returns `Error`
    /// - `WriteOnlyInterface(_)`: Returns `Error`
    /// - `IncompatibleAction(_, _)`: Returns `Error`
    /// - `WriteError(lvl, _)`: Returns the level specified in `lvl`
    /// - `ReadError(lvl, _)`: Returns the level specified in `lvl`
    ///
    /// # Returns
    ///
    /// A `HalErrorLevel` enum value, which indicates the severity of the error occurring.
    ///
    pub fn severity(&self) -> HalErrorLevel {
        match self {
            HalAlreadyLocked => Error,
            HalNotLocked => Error,
            InterfaceInitError(_) => Fatal,
            InterfaceNotFound(_) => Critical,
            WrongInterfaceId(_) => Critical,
            ReadOnlyInterface(_) => Error,
            WriteOnlyInterface(_) => Error,
            IncompatibleAction(_, _) => Error,
            WriteError(lvl, _) => *lvl,
            ReadError(lvl, _) => *lvl,
        }
    }
}
