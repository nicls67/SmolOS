#![no_std]
mod colors;
mod errors;

pub use errors::{DisplayError, DisplayErrorLevel, DisplayResult};
use hal_interface::{
    Hal, InterfaceReadAction, InterfaceWriteActions, LcdActions, LcdLayer, LcdReadAction,
};

pub use colors::Colors;
use hal_interface::InterfaceReadResult::LcdRead;
use hal_interface::LcdRead::LcdSize;

pub struct Display {
    hal_id: Option<usize>,
    hal: Option<&'static Hal>,
    size: Option<(u16, u16)>,
    initialized: bool,
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Display {
    pub fn new() -> Self {
        Self {
            hal_id: None,
            hal: None,
            size: None,
            initialized: false,
        }
    }

    pub fn init(
        &mut self,
        lcd_name: &'static str,
        hal: &'static mut Hal,
        background_color: Colors,
    ) -> DisplayResult<()> {
        // Get LCD interface ID
        self.hal_id = Some(
            hal.get_interface_id(lcd_name)
                .map_err(DisplayError::HalError)?,
        );

        // Enable display
        hal.interface_write(
            self.hal_id.unwrap(),
            InterfaceWriteActions::Lcd(LcdActions::Enable(true)),
        )
        .map_err(DisplayError::HalError)?;

        // Clear display
        hal.interface_write(
            self.hal_id.unwrap(),
            InterfaceWriteActions::Lcd(LcdActions::Clear(
                LcdLayer::FOREGROUND,
                background_color.to_argb(),
            )),
        )
        .map_err(DisplayError::HalError)?;

        // Get screen size
        self.size = match hal
            .interface_read(
                self.hal_id.unwrap(),
                InterfaceReadAction::LcdRead(LcdReadAction::LcdSize),
            )
            .map_err(DisplayError::HalError)?
        {
            LcdRead(LcdSize(x, y)) => Some((x, y)),
        };

        self.hal = Some(hal);
        self.initialized = true;
        Ok(())
    }
}
