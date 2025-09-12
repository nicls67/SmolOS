use crate::KernelError::{
    AppInitError, CannotAddNewPeriodicApp, HalError, TerminalError, WrongSyscallArgs,
};
use crate::KernelErrorLevel::{Critical, Error, Fatal};
use hal_interface::{HalError as HalErrorDef, HalErrorLevel};
use heapless::{String, format};

pub type KernelResult<T> = Result<T, KernelError>;

#[derive(Debug, Clone, Copy)]
pub enum KernelErrorLevel {
    Fatal,
    Critical,
    Error,
}

impl KernelErrorLevel {
    pub fn as_str(&self) -> &str {
        match self {
            Fatal => "Fatal error : ",
            Critical => "Critical error : ",
            Error => "Error : ",
        }
    }
}

#[derive(Debug)]
pub enum KernelError {
    HalError(HalErrorDef),
    TerminalError(KernelErrorLevel, &'static str, &'static str),
    CannotAddNewPeriodicApp(String<32>),
    AppInitError(String<32>),
    WrongSyscallArgs(&'static str),
}

impl KernelError {
    pub fn to_string(&self) -> String<256> {
        let mut msg = String::new();
        match self {
            HalError(e) => msg.push_str(e.to_string().as_str()).unwrap(),
            TerminalError(_, name, err) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(200; "Error in terminal {} : {}", name, err)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            CannotAddNewPeriodicApp(name) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(200; "Cannot add periodic app {} : app vector is full", name)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            AppInitError(name) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(200; "Cannot initialize app {}", name)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            KernelError::WrongSyscallArgs(err) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(200; "Wrong syscall arguments : {}", err)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
        }
        msg
    }

    /// Returns the severity level of the current error.
    ///
    /// # Description
    /// This method evaluates the specific type of error instance and determines
    /// its corresponding severity level as a `KernelErrorLevel`. The severity is
    /// classified into predefined levels (`Fatal`, `Critical`, `Error`), which
    /// help in categorizing and handling errors appropriately.
    ///
    /// # Behavior
    ///
    /// - For `HalError`, it delegates the severity calculation to the `HalError`
    ///   instance by mapping its `HalErrorLevel` (`Fatal`, `Critical`, `Error`) to
    ///   the corresponding `KernelErrorLevel`.
    /// - For `TerminalError`, the severity is directly returned as stored in the
    ///   error's fields.
    /// - For `CannotAddNewPeriodicApp`, the severity is always `Critical`.
    /// - For `AppInitError`, the severity is always `Critical`.
    /// - For `WrongSyscallArgs`, the severity is always `Error`.
    ///
    /// # Returns
    /// A value of `KernelErrorLevel` indicating the error severity.
    ///
    pub fn severity(&self) -> KernelErrorLevel {
        match self {
            HalError(err) => match err.severity() {
                HalErrorLevel::Fatal => Fatal,
                HalErrorLevel::Critical => Critical,
                HalErrorLevel::Error => Error,
            },
            TerminalError(lvl, _, _) => *lvl,
            CannotAddNewPeriodicApp(_) => Critical,
            AppInitError(_) => Critical,
            WrongSyscallArgs(_) => Error,
        }
    }
}
