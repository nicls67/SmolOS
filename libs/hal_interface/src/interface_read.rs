use crate::bindings::{HalInterfaceResult, get_lcd_size};

#[derive(Debug, Clone, Copy)]
pub enum InterfaceReadAction {
    LcdRead(LcdReadAction),
}

impl InterfaceReadAction {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            InterfaceReadAction::LcdRead(_) => "LCD Read",
        }
    }
}

pub enum InterfaceReadResult {
    LcdRead(LcdRead),
}

#[derive(Debug, Clone, Copy)]
pub enum LcdReadAction {
    LcdSize,
}

pub enum LcdRead {
    LcdSize(u16, u16),
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
        }
        result
    }
}
