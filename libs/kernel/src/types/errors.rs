use crate::KernelError::{AppInitError, CannotAddNewPeriodicApp, HalError, TerminalError};
use crate::KernelErrorLevel::{Critical, Error, Fatal};
use hal_interface::HalError as HalErrorDef;
use heapless::{String, format};

pub type KernelResult<T> = Result<T, KernelError>;

#[derive(Debug)]
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
}

impl KernelError {
    pub fn to_string(&self) -> String<256> {
        let mut msg = String::new();
        match self {
            HalError(e) => msg.push_str(e.to_string().as_str()).unwrap(),
            TerminalError(lvl, name, err) => {
                msg.push_str(lvl.as_str()).unwrap();
                msg.push_str(
                    format!(200; "Error in terminal {} : {}", name, err)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            CannotAddNewPeriodicApp(name) => {
                msg.push_str(Critical.as_str()).unwrap();
                msg.push_str(
                    format!(200; "Cannot add periodic app {} : app vector is full", name)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
            AppInitError(name) => {
                msg.push_str(Critical.as_str()).unwrap();
                msg.push_str(
                    format!(200; "Cannot initialize app {}", name)
                        .unwrap()
                        .as_str(),
                )
                .unwrap();
            }
        }
        msg
    }
}
