use crate::InterfaceWriteActions::{GpioWrite, Lcd, UartWrite};
use crate::LcdActions::{Clear, DrawPixel, Enable, SetFbAddress};
use crate::UartWriteActions::{SendChar, SendString};
use crate::bindings::{
    HalInterfaceResult, lcd_clear, lcd_draw_pixel, lcd_enable, set_fb_address, usart_write,
};

#[derive(Debug, Clone, Copy)]
pub enum InterfaceWriteActions<'a> {
    GpioWrite(GpioWriteAction),
    UartWrite(UartWriteActions<'a>),
    Lcd(LcdActions),
}

impl InterfaceWriteActions<'_> {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            GpioWrite(_) => "GPIO Write",
            UartWrite(_) => "UART Write",
            Lcd(_) => "LCD Write",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UartWriteActions<'a> {
    SendChar(u8),
    SendString(&'a str),
}

impl UartWriteActions<'_> {
    pub(crate) fn action(&self, id: u8) -> HalInterfaceResult {
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

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum LcdLayer {
    BACKGROUND = 0,
    FOREGROUND = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LcdPixel {
    pub x: u16,
    pub y: u16,
    pub color: PixelColorARGB,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PixelColorARGB {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PixelColorARGB {
    pub fn as_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn from_u32(color: u32) -> Self {
        PixelColorARGB {
            a: ((color >> 24) & 0xFF) as u8,
            r: ((color >> 16) & 0xFF) as u8,
            g: ((color >> 8) & 0xFF) as u8,
            b: (color & 0xFF) as u8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LcdActions {
    Enable(bool),
    Clear(LcdLayer, PixelColorARGB),
    DrawPixel(LcdLayer, LcdPixel),
    SetFbAddress(LcdLayer, u32),
}

impl LcdActions {
    pub(crate) fn action(&self, id: u8) -> HalInterfaceResult {
        match self {
            Enable(enable) => unsafe { lcd_enable(id, *enable) },
            Clear(layer, color) => unsafe { lcd_clear(id, *layer, color.as_u32()) },
            DrawPixel(layer, pixel) => unsafe {
                lcd_draw_pixel(id, *layer, pixel.x, pixel.y, pixel.color.as_u32())
            },
            SetFbAddress(layer, fb_address) => unsafe { set_fb_address(id, *layer, *fb_address) },
        }
    }
}
