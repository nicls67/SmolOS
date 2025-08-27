//! This module defines the `HalError` and `HalErrorLevel` enumerations and their associated
//! functionality. It provides a structured way to represent hardware abstraction layer (HAL)
//! related errors with different severity levels and format
use crate::HalError::{HalAlreadyLocked, InterfaceInitError};
use heapless::String;

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
    pub fn to_string(&self) -> &str {
        match self {
            HalErrorLevel::Fatal => "Fatal error : ",
            HalErrorLevel::Critical => "Critical error : ",
            HalErrorLevel::Error => "Error : ",
        }
    }
}

/// Represents errors that can occur in the HAL (Hardware Abstraction Layer).
///
/// The `HalError` enum categorizes various error types arising from operations
/// within the HAL, providing information about the error nature and severity.
///
/// # Variants
///
/// - `HalAlreadyLocked(HalErrorLevel)`
///     Indicates that the HAL is already locked, and the requested operation
///     cannot proceed. Contains a `HalErrorLevel` value that specifies the
///     severity of the error.
///
/// - `InterfaceInitError(HalErrorLevel, &'static str)`
///     Signals failure during interface initialization. Includes a `HalErrorLevel`
///     to represent error severity and a static string slice (`&'static str`)
///     providing a descriptive error message or additional context.
///

#[derive(Debug)]
pub enum HalError {
    HalAlreadyLocked(HalErrorLevel),
    InterfaceInitError(HalErrorLevel, &'static str),
}

impl HalError {
    /// Converts the current instance of an error enum into a formatted string representation.
    ///
    /// This function generates a user-friendly string message to represent the
    /// specific type of error encountered. It handles two variants of the error:
    /// `HalAlreadyLocked` and `InterfaceInitError`, formatting them appropriately.
    ///
    /// # Variants:
    /// - **HalAlreadyLocked(lvl)**: Represents the error when the Hardware Abstraction Layer (HAL)
    ///   is already locked. The `lvl` parameter is included to provide context about the level or
    ///   module that encountered the lock.
    /// - **InterfaceInitError(lvl, err)**: Represents an error during interface initialization.
    ///   Includes the `lvl` parameter for the level/module and an additional `err` parameter
    ///   that provides more detailed error information.
    ///
    /// # Returns:
    /// A `String<256>` object containing a human-readable description of the error.
    ///
    /// # Errors:
    /// If the `String` cannot be pushed to (e.g., due to capacity limits), this function will return
    /// an error or gracefully handle the situation by not pushing further.
    ///
    /// This could produce output such as:
    /// `CriticalHAL already locked` or
    /// `CriticalInterface initialization error : example_error`
    ///
    pub fn to_string(&self) -> String<256> {
        let mut msg = String::new();
        match self {
            HalAlreadyLocked(lvl) => {
                msg.push_str(lvl.to_string()).unwrap();
                msg.push_str("HAL already locked").unwrap();
            }
            InterfaceInitError(lvl, err) => {
                msg.push_str(lvl.to_string()).unwrap();
                msg.push_str("Interface initialization error : ").unwrap();
                msg.push_str(err).unwrap_or(());
            }
        }
        msg
    }
}
