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
        let mut l_msg = String::new();
        match self {
            HalError(l_e) => l_msg.push_str(l_e.to_string().as_str()).unwrap(),
            DisplayError(l_e) => l_msg.push_str(l_e.to_string().as_str()).unwrap(),
            TerminalError(_, l_err) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(200; "Error in terminal : {}", l_err)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            CannotAddNewPeriodicApp(l_name) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(200; "Cannot add periodic app {} : app vector is full", l_name)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            AppInitError(l_name) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(200; "Cannot initialize app {}", l_name)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            WrongSyscallArgs(l_err) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(200; "Wrong syscall arguments : {}", l_err)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            AppNotScheduled(l_app_name) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(200; "Could not find app {} in scheduler", l_app_name)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            AppAlreadyScheduled(l_app_name) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(200; "App {} already exists in scheduler", l_app_name)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            AppNotFound => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(format!(200; "App does not exist").unwrap().as_str())
                    .unwrap();
            }
            DeviceLocked(l_device_name) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(200; "Device {} is locked", l_device_name)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
            DeviceNotOwned(l_device_name) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(
                        format!(200; "Device {} is not owned by caller", l_device_name)
                            .unwrap()
                            .as_str(),
                    )
                    .unwrap();
            }
        }
        l_msg
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
            HalError(l_err) => match l_err.severity() {
                HalErrorLevel::Fatal => Fatal,
                HalErrorLevel::Critical => Critical,
                HalErrorLevel::Error => Error,
            },
            DisplayError(l_err) => match l_err.severity() {
                DisplayErrorLevel::Fatal => Fatal,
                DisplayErrorLevel::Critical => Critical,
                DisplayErrorLevel::Error => Error,
            },
            TerminalError(l_lvl, _) => *l_lvl,
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
