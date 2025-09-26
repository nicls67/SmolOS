use crate::InterfaceActions::{GpioWrite, UartWrite};
use crate::UartWriteActions::{SendChar, SendString};
use crate::bindings::{HalInterfaceResult, usart_write};

#[derive(Debug, Clone, Copy)]
pub enum InterfaceActions<'a> {
    GpioWrite(GpioWriteAction),
    UartWrite(UartWriteActions<'a>),
}

impl InterfaceActions<'_> {
    pub fn name(&self) -> &'static str {
        match self {
            GpioWrite(_) => "GPIO Write",
            UartWrite(_) => "UART Write",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UartWriteActions<'a> {
    SendChar(u8),
    SendString(&'a str),
}

impl UartWriteActions<'_> {
    pub fn action(&self, id: u8) -> HalInterfaceResult {
        match self {
            SendChar(c) => {
                let data_arr = [*c];
                unsafe { usart_write(id, &data_arr as *const u8, 1) }
            }
            SendString(str) => unsafe {
                usart_write(id, str.as_bytes().as_ptr(), str.len() as u16)
            },
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum GpioWriteAction {
    Set = 0,
    Clear = 1,
    Toggle = 2,
}
