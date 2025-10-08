#![no_std]
mod colors;
mod errors;
mod fonts;

pub use errors::{DisplayError, DisplayErrorLevel, DisplayResult};
pub use fonts::FontSize;
use hal_interface::{
    Hal, InterfaceReadAction, InterfaceWriteActions, LcdActions, LcdLayer, LcdPixel, LcdReadAction,
};

pub use colors::Colors;
use hal_interface::InterfaceReadResult::LcdRead;
use hal_interface::LcdRead::LcdSize;

pub struct Display {
    hal_id: Option<usize>,
    hal: Option<&'static mut Hal>,
    size: Option<(u16, u16)>,
    initialized: bool,
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Display {
    /// Creates a new instance of the struct with default values.
    ///
    /// # Returns
    /// A new instance of the struct with the following default field values:
    /// - `hal_id`: `None`
    /// - `hal`: `None`
    /// - `size`: `None`
    /// - `initialized`: `false`
    ///
    pub fn new() -> Self {
        Self {
            hal_id: None,
            hal: None,
            size: None,
            initialized: false,
        }
    }

    /// Initializes the display driver by setting up communication with the LCD hardware, enabling it,
    /// clearing the display with a specified background color, and retrieving the screen dimensions.
    ///
    /// # Parameters
    /// - `lcd_name`: A string slice that specifies the name or ID of the LCD interface.
    /// - `hal`: A mutable reference to the hardware abstraction layer (`Hal`) used for communicating
    ///   with the display interface.
    /// - `background_color`: A `Colors` enum specifying the color to fill the background of the LCD
    ///   after initialization.
    ///
    /// # Returns
    /// - `Ok(())` if the initialization is successful.
    /// - `Err(DisplayError)` if there is an issue with any of the HAL operations, such as retrieving
    ///   the LCD interface ID, enabling the display, clearing it, or reading the screen size.
    ///
    /// # Behavior
    /// - Retrieves the interface ID associated with the specified `lcd_name` from the provided HAL instance.
    /// - Enables the LCD display.
    /// - Clears the foreground layer of the display with
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

    pub fn draw_string(
        &mut self,
        string: &str,
        x: u16,
        y: u16,
        color: Colors,
        font_size: FontSize,
    ) -> DisplayResult<()> {
        // Returns error if not initialized
        if !self.initialized {
            return Err(DisplayError::DisplayDriverNotInitialized);
        }

        // Initialize variables
        let char_size = font_size.get_char_size();
        let mut current_x = x;
        let mut current_y = y;
        let color_argb = color.to_argb();

        for char_to_display in string.as_bytes() {
            // Display chat at the current position
            for line in 0..char_size.1 {
                for col in 0..char_size.0 {
                    if font_size.is_pixel_set(*char_to_display, col, line) {
                        self.hal
                            .as_mut()
                            .unwrap()
                            .interface_write(
                                self.hal_id.unwrap(),
                                InterfaceWriteActions::Lcd(LcdActions::DrawPixel(
                                    LcdLayer::FOREGROUND,
                                    LcdPixel {
                                        x: current_x + col as u16,
                                        y: current_y + line as u16,
                                        color: color_argb,
                                    },
                                )),
                            )
                            .map_err(DisplayError::HalError)?;
                    }
                }
            }

            // Compute next chat position
            current_x += char_size.0 as u16;
        }

        Ok(())
    }
}
