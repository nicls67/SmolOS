//! This module defines the `HalError` and `HalErrorLevel` enumerations and their associated
//! functionality. It provides a structured way to represent hardware abstraction layer (HAL)
//! related errors with different severity levels and format

use crate::HalError::{
    HalAlreadyInitialized, IncompatibleAction, InterfaceAlreadyLocked, InterfaceBadConfig,
    InterfaceNotFound, LockedInterface, LockerAlreadyConfigured, ReadError, ReadOnlyInterface,
    UnknownError, WriteError, WriteOnlyInterface, WrongInterfaceId,
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
    HalAlreadyInitialized,
    InterfaceNotFound(&'static str),
    WrongInterfaceId(usize),
    ReadOnlyInterface(&'static str),
    WriteOnlyInterface(&'static str),
    IncompatibleAction(&'static str, &'static str),
    WriteError(&'static str),
    ReadError(&'static str),
    LockedInterface(&'static str),
    InterfaceAlreadyLocked(&'static str),
    LockerAlreadyConfigured,
    InterfaceBadConfig(&'static str, &'static str),
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
        let mut l_msg = String::new();
        match self {
            HalAlreadyInitialized => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg.push_str("HAL already initialized").unwrap();
            }
            InterfaceNotFound(l_name) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(30; "Interface {} not found", l_name)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            WrongInterfaceId(l_id) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(30; "Interface ID {} does not exist", l_id)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            ReadOnlyInterface(l_name) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(30; "Interface {} is read-only", l_name)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            WriteOnlyInterface(l_name) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(30; "Interface {} is write-only", l_name)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            IncompatibleAction(l_action, l_interface) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg.push_str(
                    format!(70; "Action {} is not compatible with interface {}", l_action, l_interface)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            WriteError(l_ift) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(256; "Error during write on interface {} ", l_ift)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            ReadError(l_ift) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(256; "Error during read on interface {}", l_ift)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            UnknownError => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(format!(256; "Unknown HAL error").unwrap().as_str())
                    .unwrap();
            }
            LockedInterface(l_ift) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(256; "Interface {} is locked", l_ift)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            InterfaceAlreadyLocked(l_ift) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(256; "Interface {} is locked by another app", l_ift)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            LockerAlreadyConfigured => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(256; "Locker is already configured")
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            InterfaceBadConfig(l_ift, l_err) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(256; "Wrong configuration for interface {}: {}", l_ift, l_err)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
        }
        l_msg
    }

    /// Returns the severity level of the `HalError` instance.
    ///
    /// This method analyzes the type of the `HalError` and maps it to a corresponding
    /// `HalErrorLevel`, which represents how critical the error is. The mapping for
    /// each specific error variant to its respective severity level is defined.
    ///
    /// # Returns
    ///
    /// A `HalErrorLevel` enum value, which indicates the severity of the error occurring.
    ///
    pub fn severity(&self) -> HalErrorLevel {
        match self {
            HalAlreadyInitialized => Critical,
            InterfaceNotFound(_) => Critical,
            WrongInterfaceId(_) => Critical,
            ReadOnlyInterface(_) => Error,
            WriteOnlyInterface(_) => Error,
            IncompatibleAction(_, _) => Error,
            WriteError(_) => Error,
            ReadError(_) => Error,
            UnknownError => Error,
            LockedInterface(_) => Critical,
            InterfaceAlreadyLocked(_) => Critical,
            LockerAlreadyConfigured => Error,
            InterfaceBadConfig(_, _) => Critical,
        }
    }
}
