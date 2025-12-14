use crate::KernelError::{
    AppAlreadyScheduled, AppInitError, AppNotFound, AppNotScheduled, CannotAddNewPeriodicApp,
    DeviceLocked, DeviceNotOwned, DisplayError, HalError, TerminalError, WrongSyscallArgs,
};
use crate::KernelErrorLevel::{Critical, Error, Fatal};
use display::{DisplayError as DisplayErrorDef, DisplayErrorLevel};
use hal_interface::{HalError as HalErrorDef, HalErrorLevel};
use heapless::{String, format};

pub type KernelResult<T> = Result<T, KernelError>;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum KernelErrorLevel {
    Error,
    Critical,
    Fatal,
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
    DisplayError(DisplayErrorDef),
    TerminalError(KernelErrorLevel, &'static str),
    CannotAddNewPeriodicApp(&'static str),
    AppInitError(&'static str),
    WrongSyscallArgs(&'static str),
    AppNotScheduled(&'static str),
    AppAlreadyScheduled(&'static str),
    AppNotFound,
    DeviceLocked(&'static str),
    DeviceNotOwned(&'static str),
}

impl KernelError {
    pub fn to_string(&self) -> String<256> {
        let mut msg = String::new();
        match self {
            HalError(e) => msg.push_str(e.to_string().as_str()).unwrap(),
            DisplayError(e) => msg.push_str(e.to_string().as_str()).unwrap(),
            TerminalError(_, err) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(200; "Error in terminal : {}", err)
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
            AppNotScheduled(app_name) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(200; "Could not find app {} in scheduler", app_name)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            AppAlreadyScheduled(app_name) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(200; "App {} already exists in scheduler", app_name)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            AppNotFound => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(format!(200; "App does not exist").unwrap().as_str())
                    .unwrap();
            }
            DeviceLocked(device_name) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(200; "Device {} is locked", device_name)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            DeviceNotOwned(device_name) => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str(
                    format!(200; "Device {} is not owned by caller", device_name)
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
    pub fn severity(&self) -> KernelErrorLevel {
        match self {
            HalError(err) => match err.severity() {
                HalErrorLevel::Fatal => Fatal,
                HalErrorLevel::Critical => Critical,
                HalErrorLevel::Error => Error,
            },
            DisplayError(err) => match err.severity() {
                DisplayErrorLevel::Fatal => Fatal,
                DisplayErrorLevel::Critical => Critical,
                DisplayErrorLevel::Error => Error,
            },
            TerminalError(lvl, _) => *lvl,
            CannotAddNewPeriodicApp(_) => Critical,
            AppInitError(_) => Critical,
            WrongSyscallArgs(_) => Error,
            AppNotScheduled(_) => Error,
            AppAlreadyScheduled(_) => Error,
            AppNotFound => Error,
            DeviceLocked(_) => Error,
            DeviceNotOwned(_) => Error,
        }
    }
}
