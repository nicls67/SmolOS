use crate::KernelError::{HalError, TerminalError};
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
            KernelErrorLevel::Fatal => "Fatal error : ",
            KernelErrorLevel::Critical => "Critical error : ",
            KernelErrorLevel::Error => "Error : ",
        }
    }
}

#[derive(Debug)]
pub enum KernelError {
    HalError(HalErrorDef),
    TerminalError(KernelErrorLevel, &'static str, &'static str),
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
        }
        msg
    }
}
