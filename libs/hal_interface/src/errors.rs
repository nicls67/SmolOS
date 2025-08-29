//! This module defines the `HalError` and `HalErrorLevel` enumerations and their associated
//! functionality. It provides a structured way to represent hardware abstraction layer (HAL)
//! related errors with different severity levels and format
use crate::HalError::{
    HalAlreadyLocked, HalNotLocked, IncompatibleAction, InterfaceInitError, InterfaceNotFound,
    ReadOnlyInterface, WriteOnlyInterface, WrongInterfaceId,
};
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
#[derive(Debug)]
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
            HalErrorLevel::Fatal => "Fatal error : ",
            HalErrorLevel::Critical => "Critical error : ",
            HalErrorLevel::Error => "Error : ",
        }
    }
}

#[derive(Debug)]
pub enum HalError {
    HalAlreadyLocked(HalErrorLevel),
    HalNotLocked(HalErrorLevel),
    InterfaceInitError(HalErrorLevel, &'static str),
    InterfaceNotFound(HalErrorLevel, &'static str),
    WrongInterfaceId(HalErrorLevel, usize),
    ReadOnlyInterface(HalErrorLevel, &'static str),
    WriteOnlyInterface(HalErrorLevel, &'static str),
    IncompatibleAction(HalErrorLevel, &'static str, &'static str),
}

impl HalError {
    pub fn to_string(&self) -> String<256> {
        let mut msg = String::new();
        match self {
            HalAlreadyLocked(lvl) => {
                msg.push_str(lvl.as_str()).unwrap();
                msg.push_str("HAL already locked").unwrap();
            }
            HalNotLocked(lvl) => {
                msg.push_str(lvl.as_str()).unwrap();
                msg.push_str("HAL not locked").unwrap();
            }
            InterfaceInitError(lvl, err) => {
                msg.push_str(lvl.as_str()).unwrap();
                msg.push_str("Interface initialization error : ").unwrap();
                msg.push_str(err).unwrap_or(());
            }
            InterfaceNotFound(lvl, name) => {
                msg.push_str(lvl.as_str()).unwrap();
                msg.push_str(
                    format!(30; "Interface {} not found", name)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            WrongInterfaceId(lvl, id) => {
                msg.push_str(lvl.as_str()).unwrap();
                msg.push_str(
                    format!(30; "Interface ID {} does not exist", id)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            ReadOnlyInterface(lvl, name) => {
                msg.push_str(lvl.as_str()).unwrap();
                msg.push_str(
                    format!(30; "Interface {} is read-only", name)
                        .unwrap()
                        .as_str(),
                )
                    .unwrap();
            }
            WriteOnlyInterface(lvl, name) => {
                msg.push_str(lvl.as_str()).unwrap();
                msg.push_str(
                    format!(30; "Interface {} is write-only", name)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            IncompatibleAction(lvl, action, interface) => {
                msg.push_str(lvl.as_str()).unwrap();
                msg.push_str(
                    format!(70; "Action {} is not compatible with interface {}", action, interface)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
        }
        msg
    }
}
