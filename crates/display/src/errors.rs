use crate::DisplayError::HalError;
use crate::DisplayErrorLevel::{Critical, Error, Fatal};
use hal_interface::{HalError as HalErrorDef, HalErrorLevel};
use heapless::{String, format};

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
    OutOfScreenBounds,
    UnknownCharacter(u8),
    UnknownError,
}

impl DisplayError {
    pub fn to_string(&self) -> String<256> {
        let mut l_msg = String::new();
        match self {
            HalError(l_e) => l_msg.push_str(l_e.to_string().as_str()).unwrap(),
            DisplayError::DisplayDriverNotInitialized => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg.push_str("Display driver not initialized").unwrap()
            }
            DisplayError::UnknownError => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg.push_str("Unknown error").unwrap()
            }
            DisplayError::OutOfScreenBounds => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg.push_str("Out of screen bounds").unwrap()
            }
            DisplayError::UnknownCharacter(l_c) => {
                l_msg.push_str(self.severity().as_str()).unwrap();
                l_msg
                    .push_str(format!(25; "Unknown character: {}", l_c).unwrap().as_str())
                    .unwrap()
            }
        }
        l_msg
    }

    pub fn severity(&self) -> DisplayErrorLevel {
        match self {
            HalError(l_err) => match l_err.severity() {
                HalErrorLevel::Fatal => Fatal,
                HalErrorLevel::Critical => Critical,
                HalErrorLevel::Error => Error,
            },
            DisplayError::DisplayDriverNotInitialized => Error,
            DisplayError::UnknownError => Error,
            DisplayError::OutOfScreenBounds => Error,
            DisplayError::UnknownCharacter(_) => Error,
        }
    }
}
