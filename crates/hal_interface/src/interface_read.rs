use crate::K_BUFFER_SIZE;
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
    BufferRead(Vec<u8, K_BUFFER_SIZE>),
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
