use crate::BUFFER_SIZE;
use crate::LcdLayer;
use crate::bindings::{HalInterfaceResult, get_fb_address, get_lcd_size};
use heapless::Vec;

#[repr(C)]
#[derive(Clone)]
pub(crate) struct RxBuffer {
    pub buffer: *mut u8,
    pub size: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum InterfaceReadAction {
    LcdRead(LcdReadAction),
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

pub enum InterfaceReadResult {
    LcdRead(LcdRead),
    BufferRead(Vec<u8, BUFFER_SIZE>),
}

#[derive(Debug, Clone, Copy)]
pub enum LcdReadAction {
    LcdSize,
    FbAddress(LcdLayer),
}

pub enum LcdRead {
    LcdSize(u16, u16),
    FbAddress(u32),
}

impl LcdReadAction {
    pub(crate) fn read(&self, id: usize, read_result: &mut LcdRead) -> HalInterfaceResult {
        let result;
        match self {
            LcdReadAction::LcdSize => {
                let mut x: u16 = 0;
                let mut y: u16 = 0;
                result = unsafe { get_lcd_size(id as u8, &mut x, &mut y) };
                *read_result = LcdRead::LcdSize(x, y);
            }
            LcdReadAction::FbAddress(layer) => {
                let mut fb_address: u32 = 0;
                result = unsafe { get_fb_address(id as u8, *layer, &mut fb_address) };
                *read_result = LcdRead::FbAddress(fb_address);
            }
        }
        result
    }
}
