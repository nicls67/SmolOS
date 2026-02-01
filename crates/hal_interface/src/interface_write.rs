use crate::InterfaceWriteActions::{GpioWrite, Lcd, UartWrite};
use crate::LcdActions::{Clear, DrawPixel, Enable, SetFbAddress};
use crate::UartWriteActions::{SendChar, SendString};
use crate::bindings::{
    HalInterfaceResult, lcd_clear, lcd_draw_pixel, lcd_enable, set_fb_address, usart_write,
};

/// High-level enum representing all possible write actions on any hardware interface.
#[derive(Debug, Clone, Copy)]
pub enum InterfaceWriteActions<'a> {
    /// Write action for GPIO interfaces.
    GpioWrite(GpioWriteAction),
    /// Write action for UART interfaces.
    UartWrite(UartWriteActions<'a>),
    /// Write action for LCD interfaces.
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

/// Represents write operations specific to UART interfaces.
#[derive(Debug, Clone, Copy)]
pub enum UartWriteActions<'a> {
    /// Send a single byte.
    SendChar(u8),
    /// Send a string of bytes.
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

/// Represents possible actions on a GPIO pin.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum GpioWriteAction {
    /// Set the pin to a high state.
    Set = 0,
    /// Set the pin to a low state.
    Clear = 1,
    /// Toggle the pin state.
    Toggle = 2,
}

/// Represents the available LCD layers.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum LcdLayer {
    /// The background layer.
    BACKGROUND = 0,
    /// The foreground layer.
    FOREGROUND = 1,
}

/// Represents a pixel on the LCD screen.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LcdPixel {
    /// X coordinate in pixels.
    pub x: u16,
    /// Y coordinate in pixels.
    pub y: u16,
    /// Color of the pixel.
    pub color: PixelColorARGB,
}

/// Represents a color in ARGB format.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PixelColorARGB {
    /// Alpha component.
    pub a: u8,
    /// Red component.
    pub r: u8,
    /// Green component.
    pub g: u8,
    /// Blue component.
    pub b: u8,
}

impl PixelColorARGB {
    /// Converts the ARGB color to a `u32`.
    ///
    /// # Returns
    /// A `u32` value in 0xAARRGGBB format.
    pub fn as_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Creates a `PixelColorARGB` from a `u32` in 0xAARRGGBB format.
    ///
    /// # Parameters
    /// - `p_color`: The color as a `u32`.
    ///
    /// # Returns
    /// A new `PixelColorARGB` instance.
    pub fn from_u32(p_color: u32) -> Self {
        PixelColorARGB {
            a: ((p_color >> 24) & 0xFF) as u8,
            r: ((p_color >> 16) & 0xFF) as u8,
            g: ((p_color >> 8) & 0xFF) as u8,
            b: (p_color & 0xFF) as u8,
        }
    }
}

/// Represents possible actions on an LCD interface.
#[derive(Debug, Clone, Copy)]
pub enum LcdActions {
    /// Enable or disable the LCD display.
    Enable(bool),
    /// Clear a specific layer with a color.
    Clear(LcdLayer, PixelColorARGB),
    /// Draw a single pixel on a layer.
    DrawPixel(LcdLayer, LcdPixel),
    /// Set the base address of the frame buffer for a layer.
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
