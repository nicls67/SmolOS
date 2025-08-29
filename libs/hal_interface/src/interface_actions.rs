use crate::HalErrorLevel::Error;
use crate::InterfaceReadActions::UartRead;
use crate::InterfaceWriteActions::{GpioWrite, UartWrite};
use crate::UartReadActions::Read;
use crate::UartWriteActions::{SendChar, SendString};
use crate::async_block::block_on;
use crate::{HalError, HalResult};
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::Uart;

pub enum InterfaceWriteActions {
    GpioWrite(GpioWriteActions),
    UartWrite(UartWriteActions),
}

impl InterfaceWriteActions {
    pub fn name(&self) -> &'static str {
        match self {
            GpioWrite(_) => "GPIO Write",
            UartWrite(_) => "Uart Write",
        }
    }
}

pub enum GpioWriteActions {
    Set,
    Clear,
    Toggle,
}

impl GpioWriteActions {
    pub fn action(&self, pin: &mut Output) -> HalResult<()> {
        match self {
            GpioWriteActions::Set => {
                pin.set_high();
                Ok(())
            }
            GpioWriteActions::Clear => {
                pin.set_low();
                Ok(())
            }
            GpioWriteActions::Toggle => {
                pin.toggle();
                Ok(())
            }
        }
    }
}

pub enum UartWriteActions {
    SendChar(u8),
    SendString(&'static str),
}

impl UartWriteActions {
    pub fn action(&self, uart: &mut Uart<'static, Async>) -> HalResult<()> {
        match self {
            SendChar(c) => {
                let data_arr = [*c];
                block_on(uart.write(&data_arr)).map_err(|_| HalError::WriteError(Error, "UART"))
            }
            SendString(str) => block_on(uart.write(str.as_bytes()))
                .map_err(|_| HalError::WriteError(Error, "UART")),
        }
    }
}

pub enum InterfaceReadActions<'a> {
    UartRead(UartReadActions<'a>),
}

impl InterfaceReadActions<'_> {
    pub fn name(&self) -> &'static str {
        match self {
            UartRead(_) => "UART Read",
        }
    }
}

pub enum UartReadActions<'a> {
    Read(&'a mut [u8]),
}

impl UartReadActions<'_> {
    pub fn action(&mut self, uart: &mut Uart<'static, Async>) -> HalResult<()> {
        match self {
            Read(buffer) => {
                block_on(uart.read(buffer)).map_err(|_| HalError::ReadError(Error, "UART"))
            }
        }
    }
}
