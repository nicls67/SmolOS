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
    pub(crate) fn action(&self, p_id: u8) -> HalInterfaceResult {
        match self {
            SendChar(l_c) => {
                let l_data_arr = [*l_c];
                unsafe { usart_write(p_id, &l_data_arr as *const u8, 1) }
            }
            SendString(l_str) => unsafe {
                usart_write(p_id, l_str.as_bytes().as_ptr(), l_str.len() as u16)
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

    pub fn from_u32(p_color: u32) -> Self {
        PixelColorARGB {
            a: ((p_color >> 24) & 0xFF) as u8,
            r: ((p_color >> 16) & 0xFF) as u8,
            g: ((p_color >> 8) & 0xFF) as u8,
            b: (p_color & 0xFF) as u8,
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
    pub(crate) fn action(&self, p_id: u8) -> HalInterfaceResult {
        match self {
            Enable(l_enable) => unsafe { lcd_enable(p_id, *l_enable) },
            Clear(l_layer, l_color) => unsafe { lcd_clear(p_id, *l_layer, l_color.as_u32()) },
            DrawPixel(l_layer, l_pixel) => unsafe {
                lcd_draw_pixel(p_id, *l_layer, l_pixel.x, l_pixel.y, l_pixel.color.as_u32())
            },
            SetFbAddress(l_layer, l_fb_address) => unsafe {
                set_fb_address(p_id, *l_layer, *l_fb_address)
            },
        }
    }
}
