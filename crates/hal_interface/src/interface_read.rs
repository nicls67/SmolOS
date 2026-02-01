use crate::K_BUFFER_SIZE;
use crate::LcdLayer;
use crate::bindings::{HalInterfaceResult, get_fb_address, get_lcd_size};
use heapless::Vec;

/// Represents a raw receive buffer used by the underlying C HAL.
#[repr(C)]
#[derive(Clone)]
pub(crate) struct RxBuffer {
    /// Pointer to the raw data buffer.
    pub buffer: *mut u8,
    /// Number of bytes currently in the buffer.
    pub size: u8,
}

/// Represents possible read actions on any hardware interface.
#[derive(Debug, Clone, Copy)]
pub enum InterfaceReadAction {
    /// Read action specific to LCD interfaces.
    LcdRead(LcdReadAction),
    /// Read action for interfaces with a receive buffer (e.g., UART).
    BufferRead,
}

impl InterfaceReadAction {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            InterfaceReadAction::LcdRead(_) => "LCD Read",
            InterfaceReadAction::BufferRead => "Buffer Read",
        }
    }
}

/// Encapsulates the result of a hardware interface read operation.
pub enum InterfaceReadResult {
    /// Result of an LCD read operation.
    LcdRead(LcdRead),
    /// Data read from a receive buffer.
    BufferRead(Vec<u8, K_BUFFER_SIZE>),
}

/// Specific read operations for LCD interfaces.
#[derive(Debug, Clone, Copy)]
pub enum LcdReadAction {
    /// Read the screen dimensions (width, height).
    LcdSize,
    /// Read the frame buffer base address for a specific layer.
    FbAddress(LcdLayer),
}

/// Data returned from LCD read operations.
pub enum LcdRead {
    /// Screen size as (width, height).
    LcdSize(u16, u16),
    /// Frame buffer memory address.
    FbAddress(u32),
}

impl LcdReadAction {
    pub(crate) fn read(&self, p_id: usize, p_read_result: &mut LcdRead) -> HalInterfaceResult {
        let l_result;
        match self {
            LcdReadAction::LcdSize => {
                let mut l_x: u16 = 0;
                let mut l_y: u16 = 0;
                l_result = unsafe { get_lcd_size(p_id as u8, &mut l_x, &mut l_y) };
                *p_read_result = LcdRead::LcdSize(l_x, l_y);
            }
            LcdReadAction::FbAddress(l_layer) => {
                let mut l_fb_address: u32 = 0;
                l_result = unsafe { get_fb_address(p_id as u8, *l_layer, &mut l_fb_address) };
                *p_read_result = LcdRead::FbAddress(l_fb_address);
            }
        }
        l_result
    }
}
