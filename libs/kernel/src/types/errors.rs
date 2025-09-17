use crate::KernelError::{
    AppInitError, AppNotFound, CannotAddNewPeriodicApp, HalError, TerminalError, WrongSyscallArgs,
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
    CannotAddNewPeriodicApp(&'static str),
    AppInitError(&'static str),
    WrongSyscallArgs(&'static str),
    AppNotFound(&'static str),
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
            WrongSyscallArgs(err) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(200; "Wrong syscall arguments : {}", err)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            AppNotFound(app_name) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(200; "Could not find app {} in scheduler", app_name)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
        }
        msg
    }

    /// Returns the severity level of the kernel error.
    ///
    /// This method evaluates the severity of the error
    /// based on its specific type. The returned severity
    /// is conveyed as a `KernelErrorLevel` enum, which can
    /// represent `Fatal`, `Critical`, or `Error` levels.
    ///
    /// # Behavior
    /// - For `HalError`, the severity is determined based on
    ///   the severity of the associated `HalErrorLevel`.
    ///   - `HalErrorLevel::Fatal` maps to `KernelErrorLevel::Fatal`.
    ///   - `HalErrorLevel::Critical` maps to `KernelErrorLevel::Critical`.
    ///   - `HalErrorLevel::Error` maps to `KernelErrorLevel::Error`.
    /// - For `TerminalError`, the severity level is directly extracted from the
    ///   first tuple field.
    /// - For `CannotAddNewPeriodicApp`, the severity is set to `Critical`.
    /// - For `AppInitError`, the severity is set to `Critical`.
    /// - For `WrongSyscallArgs`, the severity is set to `Error`.
    /// - For `AppNotFound`, the severity is set to `Error`
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
            AppNotFound(_) => Error,
        }
    }
}
