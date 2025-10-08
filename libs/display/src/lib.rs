#![no_std]
mod colors;
mod errors;
mod fonts;
mod frame_buffer;

pub use errors::{DisplayError, DisplayErrorLevel, DisplayResult};
pub use fonts::FontSize;
use hal_interface::{
    Hal, InterfaceReadAction, InterfaceWriteActions, LcdActions, LcdLayer, LcdReadAction,
};

use crate::frame_buffer::FrameBuffer;
pub use colors::Colors;
use hal_interface::InterfaceReadResult::LcdRead;
use hal_interface::LcdRead::LcdSize;

pub struct Display {
    hal_id: Option<usize>,
    hal: Option<&'static mut Hal>,
    size: Option<(u16, u16)>,
    frame_buffer: Option<FrameBuffer>,
    initialized: bool,
    cursor_pos: (u16, u16),
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Display {
    /// Initializes and returns a new instance of the struct.
    ///
    /// # Description
    /// This function is a constructor for creating a new instance of the struct.
    /// It initializes all fields with their default values.
    ///
    /// # Returns
    /// A new instance of the struct with all fields set to their default values.
    ///
    pub fn new() -> Self {
        Self {
            hal_id: None,
            hal: None,
            size: None,
            frame_buffer: None,
            initialized: false,
            cursor_pos: (0, 0),
        }
    }

    /// Initializes the display by setting up its interface, enabling the LCD, clearing it with
    /// a specified background color, retrieving its size, and preparing the frame buffer for rendering.
    ///
    /// # Parameters
    /// - `lcd_name`: A `&'static str` representing the name/identifier of the LCD display to be initialized.
    /// - `hal`: A mutable reference to the `Hal` interface, which provides methods to communicate with the hardware layer.
    /// - `background_color`: A `Colors` enumeration value that specifies the background color to clear the display with.
    ///
    /// # Returns
    /// - `DisplayResult<()>`: Returns `Ok(())` if the initialization process completes successfully, or an error
    ///   wrapped in `DisplayResult` if any step in the initialization fails.
    ///
    /// # Errors
    /// - Returns a `DisplayError::HalError` if any interaction with the HAL
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

        // Get screen size
        self.size = match hal
            .interface_read(
                self.hal_id.unwrap(),
                InterfaceReadAction::LcdRead(LcdReadAction::LcdSize),
            )
            .map_err(DisplayError::HalError)?
        {
            LcdRead(LcdSize(x, y)) => Some((x, y)),
            _ => None,
        };

        // Store HAL reference
        self.hal = Some(hal);

        // Initialize the frame buffer
        self.frame_buffer = Some(FrameBuffer::new());

        // Clean the buffer and then switch to the other buffer
        self.hal
            .as_mut()
            .unwrap()
            .interface_write(
                self.hal_id.unwrap(),
                InterfaceWriteActions::Lcd(LcdActions::Clear(
                    LcdLayer::FOREGROUND,
                    background_color.to_argb(),
                )),
            )
            .map_err(DisplayError::HalError)?;

        self.initialized = true;

        Ok(())
    }

    /// Switches the current frame buffer and updates the display hardware with the new frame buffer address.
    ///
    /// This function switches the current frame buffer to a new one by calling the `switch`
    /// method on the existing frame buffer. Once the new frame buffer address is determined,
    /// it communicates with the hardware abstraction layer (HAL) to inform the display hardware
    /// of the new frame buffer address for the foreground layer.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the frame buffer switch and hardware update were successful.
    /// If an error occurs during a HAL interface write operation, it returns a `DisplayError::HalError`.
    ///
    /// # Errors
    ///
    /// This function may return a `DisplayError::HalError` if there is an issue writing
    /// to the HAL interface.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The `frame_buffer` field is `None`. This indicates that the frame buffer has not been properly
    /// initialized.
    /// - The `hal` field is `None`. This indicates that the hardware abstraction layer is not initialized.
    /// - The `hal_id` field is `None`. This indicates the HAL identifier is missing.
    ///
    /// In this example, the function switches the current frame buffer and updates the associated
    /// hardware accordingly.
    pub fn switch_frame_buffer(&mut self) -> DisplayResult<()> {
        let fb_addr = self.frame_buffer.as_mut().unwrap().switch();

        self.hal
            .as_mut()
            .unwrap()
            .interface_write(
                self.hal_id.unwrap(),
                InterfaceWriteActions::Lcd(LcdActions::SetFbAddress(LcdLayer::FOREGROUND, fb_addr)),
            )
            .map_err(DisplayError::HalError)?;

        Ok(())
    }

    /// Draws a string on the display at the specified position, with a given color and font size.
    ///
    /// # Parameters
    /// - `string`: A reference to the string (`&str`) to be displayed on the screen.
    /// - `x`: The x-coordinate (horizontal position) on the display where the string will start.
    /// - `y`: The y-coordinate (vertical position) on the display where the string will start.
    /// - `color`: A `Colors` object that represents the color in which the text will be displayed.
    /// - `font_size`: A `FontSize` object that determines the size/dimensions of each character.
    ///
    /// # Errors
    /// Returns an error of type `DisplayError::DisplayDriverNotInitialized` if the display driver
    /// has not been initialized before calling this method. Ensure the display is properly initialized
    /// by the time this function is invoked.
    ///
    /// # Behavior
    /// - The function iterates through each character in the input string and draws it sequentially
    ///   at the appropriate position on the display, based on the given coordinates and font size.
    /// - For each character, the method computes the pixel positions, checks the font map to determine
    ///   which pixels are set, and updates the frame buffer with the corresponding color for the active
    ///   pixels.
    ///
    /// # Notes
    /// - The function assumes a linear frame buffer addressing scheme.
    /// - The position of each drawn character advances horizontally by the character width, as defined
    ///   by the given `FontSize`.
    /// - Frame buffer addressing takes into account the display width and the dimensions of the font
    ///   characters to ensure proper placement of the string.
    ///
    /// # Returns
    /// Returns `Ok(())` if the string was drawn successfully on the display. Otherwise, it returns
    /// a `DisplayError` if an error occurred, such as the display being uninitialized.
    ///
    /// # Performance
    /// This function involves pixel-level operations for every character in the string. For large
    /// strings or high-resolution fonts, the time to render may increase significantly. Consider
    /// optimizing the frame buffer access if drawing speed is critical.
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
        let color_argb = color.to_argb().as_u32();
        let mut fb_write_address = self.frame_buffer.as_mut().unwrap().address_displayed()
            + 4 * (y as u32 * self.size.unwrap().0 as u32 + x as u32);

        for char_to_display in string.as_bytes() {
            // Display chat at the current position
            for line in 0..char_size.1 {
                for col in 0..char_size.0 {
                    if font_size.is_pixel_set(*char_to_display, col, line) {
                        unsafe {
                            *(fb_write_address as *mut u32) = color_argb;
                        }
                    }

                    // Increment frame buffer address
                    fb_write_address += 4;
                }

                // Increment frame buffer address
                fb_write_address += self.size.unwrap().0 as u32 * 4 - char_size.0 as u32 * 4;
            }

            // Compute next char position
            current_x += char_size.0 as u16;
            // Increment frame buffer address
            fb_write_address = self.frame_buffer.as_mut().unwrap().address_displayed()
                + 4 * (y as u32 * self.size.unwrap().0 as u32 + current_x as u32);
        }

        Ok(())
    }
}
