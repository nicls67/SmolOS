//! This module defines the `HalError` and `HalErrorLevel` enumerations and their associated
//! functionality. It provides a structured way to represent hardware abstraction layer (HAL)
//! related errors with different severity levels and format

use crate::HalError::{
    IncompatibleAction, InterfaceAlreadyLocked, InterfaceNotFound, LockedInterface,
    LockerAlreadyConfigured, ReadError, ReadOnlyInterface, UnknownError, WriteError,
    WriteOnlyInterface, WrongInterfaceId,
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
            Fatal => "HAL Fatal error : ",
            Critical => "HAL Critical error : ",
            Error => "HAL Error : ",
        }
    }
}

#[derive(Debug)]
pub enum HalError {
    InterfaceNotFound(&'static str),
    WrongInterfaceId(usize),
    ReadOnlyInterface(&'static str),
    WriteOnlyInterface(&'static str),
    IncompatibleAction(&'static str, &'static str),
    WriteError(HalErrorLevel, &'static str),
    ReadError(HalErrorLevel, &'static str),
    LockedInterface(&'static str),
    InterfaceAlreadyLocked(&'static str),
    LockerAlreadyConfigured,
    UnknownError,
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
            WriteError(_, ift) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(256; "Error during write on interface {} ", ift)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            ReadError(_, ift) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(256; "Error during read on interface {}", ift)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            UnknownError => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(format!(256; "Unknown HAL error").unwrap().as_str())
                    .unwrap();
            }
            LockedInterface(ift) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(256; "Interface {} is locked", ift)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            InterfaceAlreadyLocked(ift) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(256; "Interface {} is locked by another app", ift)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            LockerAlreadyConfigured => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(256; "Locker is already configured")
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
    /// - `InterfaceNotFound(_)`: Returns `Critical`
    /// - `WrongInterfaceId(_)`: Returns `Critical`
    /// - `ReadOnlyInterface(_)`: Returns `Error`
    /// - `WriteOnlyInterface(_)`: Returns `Error`
    /// - `IncompatibleAction(_, _)`: Returns `Error`
    /// - `WriteError(lvl, _)`: Returns the level specified in `lvl`
    /// - `ReadError(lvl, _)`: Returns the level specified in `lvl`
    /// - `UnknownError`: Returns `Error`
    ///
    /// # Returns
    ///
    /// A `HalErrorLevel` enum value, which indicates the severity of the error occurring.
    ///
    pub fn severity(&self) -> HalErrorLevel {
        match self {
            InterfaceNotFound(_) => Critical,
            WrongInterfaceId(_) => Critical,
            ReadOnlyInterface(_) => Error,
            WriteOnlyInterface(_) => Error,
            IncompatibleAction(_, _) => Error,
            WriteError(lvl, _) => *lvl,
            ReadError(lvl, _) => *lvl,
            UnknownError => Error,
            LockedInterface(_) => Critical,
            InterfaceAlreadyLocked(_) => Critical,
            LockerAlreadyConfigured => Error,
        }
    }
}
