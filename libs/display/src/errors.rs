use crate::DisplayError::HalError;
use crate::DisplayErrorLevel::{Critical, Error, Fatal};
use hal_interface::{HalError as HalErrorDef, HalErrorLevel};
use heapless::String;

pub type DisplayResult<T> = Result<T, DisplayError>;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum DisplayErrorLevel {
    Error,
    Critical,
    Fatal,
}

impl DisplayErrorLevel {
    pub fn as_str(&self) -> &str {
        match self {
            Fatal => "Fatal display error : ",
            Critical => "Critical display error : ",
            Error => "Display error : ",
        }
    }
}

#[derive(Debug)]
pub enum DisplayError {
    HalError(HalErrorDef),
    DisplayDriverNotInitialized,
    UnknownError,
}

impl DisplayError {
    pub fn to_string(&self) -> String<256> {
        let mut msg = String::new();
        match self {
            HalError(e) => msg.push_str(e.to_string().as_str()).unwrap(),
            DisplayError::DisplayDriverNotInitialized => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str("Display driver not initialized").unwrap()
            }
            DisplayError::UnknownError => {
                msg.push_str(self.severity().as_str()).unwrap();
                msg.push_str("Unknown error").unwrap()
            }
        }
        msg
    }

    pub fn severity(&self) -> DisplayErrorLevel {
        match self {
            HalError(err) => match err.severity() {
                HalErrorLevel::Fatal => Fatal,
                HalErrorLevel::Critical => Critical,
                HalErrorLevel::Error => Error,
            },
            DisplayError::DisplayDriverNotInitialized => Error,
            DisplayError::UnknownError => Error,
        }
    }
}
