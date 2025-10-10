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

use crate::FontSize::Font16;
use crate::fonts::{FIRST_ASCII_CHAR, LAST_ASCII_CHAR};
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
    font: FontSize,
    color: Colors,
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
            font: Font16,
            color: Colors::White,
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

        // Mark the driver as initialized
        self.initialized = true;

        // Clean the buffer
        self.clear(background_color)?;

        Ok(())
    }

    /// Clears the display's foreground layer with the specified color.
    ///
    /// # Parameters
    ///
    /// * `color` - A value of type `Colors` representing the color to fill the entire foreground layer of the display.
    ///
    /// # Returns
    ///
    /// * `DisplayResult<()>` -
    ///     * Returns `Ok(())` if the foreground layer is successfully cleared with the specified color.
    ///     * Returns an `Err(DisplayError)` if an error occurs during the process, such as
    ///         - `DisplayError::HalError` if the hardware abstraction layer (HAL) fails during the interface write operation.
    ///         - `DisplayError::DisplayDriverNotInitialized` if the display driver has not been properly initialized.
    ///
    /// # Behavior
    ///
    /// * If the display driver is initialized (`self.initialized` is true), the method writes the `Clear` action with the specified color
    ///   to the hardware abstraction layer through the driver interface (`interface_write`).
    ///
    /// * If the display driver is not initialized, the method returns a `DisplayError::DisplayDriverNotInitialized` error.
    ///
    /// # Errors
    ///
    /// * `DisplayError::DisplayDriverNotInitialized`: Thrown if the display driver has not been initialized.
    /// * `DisplayError::HalError`: Thrown if the HAL interface write operation fails.
    ///
    /// # Notes
    ///
    /// * Ensure that the display driver is initialized before calling this method.
    /// * The `self.hal` and `self.hal_id` must be `Some` values for successful operation.
    pub fn clear(&mut self, color: Colors) -> DisplayResult<()> {
        if self.initialized {
            self.hal
                .as_mut()
                .unwrap()
                .interface_write(
                    self.hal_id.unwrap(),
                    InterfaceWriteActions::Lcd(LcdActions::Clear(
                        LcdLayer::FOREGROUND,
                        color.to_argb(),
                    )),
                )
                .map_err(DisplayError::HalError)?;
            self.cursor_pos = (0, 0);
            Ok(())
        } else {
            Err(DisplayError::DisplayDriverNotInitialized)
        }
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

    /// Draws a string on the display at the specified position, with a given color.
    ///
    /// # Parameters
    /// - `string`: A reference to the string (`&str`) to be displayed on the screen.
    /// - `x`: The x-coordinate (horizontal position) on the display where the string will start.
    /// - `y`: The y-coordinate (vertical position) on the display where the string will start.
    /// - `color`: A `Colors` object that represents the color in which the text will be displayed.
    ///
    /// # Errors
    /// Returns an error of type `DisplayError::DisplayDriverNotInitialized` if the display driver
    /// has not been initialized before calling this method. Ensure the display is properly initialized
    /// by the time this function is invoked.
    ///
    /// # Behavior
    /// - The function iterates through each character in the input string and draws it sequentially
    ///   at the appropriate position on the display, based on the given coordinates.
    /// - For each character, the method computes the pixel positions, checks the font map to determine
    ///   which pixels are set, and updates the frame buffer with the corresponding color for the active
    ///   pixels.
    ///
    /// # Notes
    /// - The function assumes a linear frame buffer addressing scheme.
    /// - The position of each drawn character advances horizontally by the character width, as defined
    ///   by the selected font size.
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
        color: Option<Colors>,
    ) -> DisplayResult<()> {
        // Returns error if not initialized
        if !self.initialized {
            return Err(DisplayError::DisplayDriverNotInitialized);
        }

        // Initialize variables
        let char_size = self.font.get_char_size();
        let mut current_x = x;

        // Get display color
        let color_argb = if let Some(c) = color {
            c.to_argb().as_u32()
        } else {
            self.color.to_argb().as_u32()
        };

        // Compute frame buffer address
        let mut fb_write_address = self.frame_buffer.as_mut().unwrap().address_displayed()
            + 4 * (y as u32 * self.size.unwrap().0 as u32 + x as u32);

        for char_to_display in string.as_bytes() {
            self.draw_char_in_fb(*char_to_display, fb_write_address, char_size, color_argb)?;

            // Compute next char position
            current_x += char_size.0 as u16;
            // Increment frame buffer address
            fb_write_address = self.frame_buffer.as_mut().unwrap().address_displayed()
                + 4 * (y as u32 * self.size.unwrap().0 as u32 + current_x as u32);
        }

        Ok(())
    }

    /// Draws a single character on the display at the specified position with an optional custom color.
    ///
    /// # Parameters
    /// - `char_to_display`: The ASCII value of the character to be displayed.
    /// - `x`: The horizontal position (in pixels) where the character will be drawn.
    /// - `y`: The vertical position (in pixels) where the character will be drawn.
    /// - `color`: An optional parameter specifying the color of the character. If `None`, the default color of the display is used.
    ///
    /// # Returns
    /// - `DisplayResult<()>`: Returns `Ok(())` if the operation is successful. Returns an error if the display driver is not initialized or if there is a failure while drawing the character.
    ///
    /// # Errors
    /// - `DisplayError::DisplayDriverNotInitialized`: Returned if the display has not been initialized prior to calling this method.
    /// - Other errors propagated from the underlying `draw_char_in_fb` function.
    ///
    /// # Behavior
    /// 1. Checks if the display has been properly initialized. If not, it returns an appropriate error.
    /// 2. Uses the font information to determine the size of the character.
    /// 3. Computes the address in the frame buffer where the character's pixel data will be written.
    /// 4. Calls `draw_char_in_fb` to render the character into the frame buffer with the calculated position and color.
    ///
    /// # Note
    /// - The `frame_buffer` must be correctly set up and mutable before calling this function.
    /// - The `Colors` enum should provide an `to_argb` method that converts the color to an ARGB format.
    ///
    pub fn draw_char(
        &mut self,
        char_to_display: u8,
        x: u16,
        y: u16,
        color: Option<Colors>,
    ) -> DisplayResult<()> {
        // Returns error if not initialized
        if !self.initialized {
            return Err(DisplayError::DisplayDriverNotInitialized);
        }

        let char_size = self.font.get_char_size();

        // Get display color
        let color_argb = if let Some(c) = color {
            c.to_argb().as_u32()
        } else {
            self.color.to_argb().as_u32()
        };

        // Compute frame buffer address
        let fb_write_address = self.frame_buffer.as_mut().unwrap().address_displayed()
            + 4 * (y as u32 * self.size.unwrap().0 as u32 + x as u32);

        // Draw char in fb
        self.draw_char_in_fb(char_to_display, fb_write_address, char_size, color_argb)?;

        Ok(())
    }

    ///
    /// Draws a character onto the frame buffer at the specified location with the given size and color.
    ///
    /// # Parameters
    /// - `char_to_display`: The ASCII value of the character to be displayed.
    ///     * Special handling is applied for:
    ///         - `b'\n'`: Moves the cursor to a new line.
    ///         - `b'\r'`: Ignored during rendering.
    /// - `fb_write_address`: The memory address in the frame buffer where the character drawing starts.
    /// - `char_size`: A tuple specifying the dimensions of the character `(width, height)` in pixels.
    /// - `color_argb`: The color of the character in 32-bit ARGB format.
    ///
    /// # Returns
    /// A `DisplayResult<()>` which is `Ok` on successful completion or contains an error if something goes wrong.
    ///
    /// # Behavior
    /// - If the character is a newline (`b'\n'`), the cursor position is updated to a new line using `self.set_cursor_at_new_line()`.
    /// - If the character is not a carriage return (`b'\r'`), the function iteratively checks the corresponding bitmap
    ///   for the character using `self.font.is_pixel_set()`. If a pixel is set, it writes the `color_argb` value
    ///   at the position in the frame buffer defined by `fb_write_address`.
    /// - The frame buffer's address is incremented as necessary to step through pixels or rows after drawing.
    ///
    /// # Safety
    /// - Directly writes to the frame buffer through unsafe raw pointer dereferencing. Ensure valid memory address
    fn draw_char_in_fb(
        &mut self,
        char_to_display: u8,
        mut fb_write_address: u32,
        char_size: (u8, u8),
        color_argb: u32,
    ) -> DisplayResult<()> {
        // Check if the character to display is valid
        if !(FIRST_ASCII_CHAR..=LAST_ASCII_CHAR).contains(&char_to_display) {
            return Err(DisplayError::UnknownCharacter(char_to_display));
        } else {
            // Display chat at the current position
            for line in 0..char_size.1 {
                for col in 0..char_size.0 {
                    if self.font.is_pixel_set(char_to_display, col, line) {
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
        }

        Ok(())
    }

    /// Draws a provided string at the current cursor position on the display.
    ///
    /// This method takes a string slice `string` and iterates through its characters,
    /// drawing each character one by one at the current cursor position.
    /// If a color is provided, it will be used to set the color of the characters. If `color`
    /// is `None`, the default color will be used.
    ///
    /// The method automatically handles advancing the cursor position after rendering
    /// each character.
    ///
    /// # Arguments
    ///
    /// * `string` - A string slice containing the characters to be rendered.
    /// * `color` - An optional parameter specifying the color of the string. If `None`,
    ///   the default color for the display will be applied.
    ///
    /// # Returns
    ///
    /// A `DisplayResult<()>` which indicates success. If drawing any individual
    /// character fails, it will return an error contained within the `DisplayResult`.
    ///
    /// # Errors
    ///
    /// The method will propagate any errors returned while drawing individual characters,
    /// allowing the caller to handle such cases. Errors could arise from display rendering
    /// issues or cursor operation failures.
    ///
    pub fn draw_string_at_cursor(
        &mut self,
        string: &str,
        color: Option<Colors>,
    ) -> DisplayResult<()> {
        // Draw the string at the current cursor position
        for char_to_display in string.as_bytes() {
            self.draw_char_at_cursor(*char_to_display, color)?;
        }
        Ok(())
    }

    /// Draws a character at the current cursor position on the display.
    ///
    /// This function places a given character (`char_to_display`) at the
    /// current cursor position and optionally applies a specified color.
    ///
    /// Special handling is applied for newline (`b'\n'`) and carriage return (`b'\r'`) characters:
    /// - For `b'\n'`, the cursor is moved to the next line (line feed).
    /// - For `b'\r'`, the cursor is moved back to the beginning of the current line (carriage return).
    /// - For all other characters, the character is drawn at the current cursor position, and then
    ///   the cursor is moved forward.
    ///
    /// # Parameters
    /// - `char_to_display`: A `u8` representing the ASCII value of the character to draw.
    /// - `color`: An optional `Colors` value that specifies the color of the character.
    ///
    /// # Returns
    /// - `DisplayResult<()>`: Returns `Ok(())` on success, or an error if drawing or cursor movement fails.
    ///
    /// # Errors
    /// This function returns an error if:
    /// - The character drawing operation fails.
    /// - Moving the cursor fails.
    ///
    pub fn draw_char_at_cursor(
        &mut self,
        char_to_display: u8,
        color: Option<Colors>,
    ) -> DisplayResult<()> {
        if char_to_display == b'\n' {
            self.set_cursor_line_feed()?;
        } else if char_to_display == b'\r' {
            self.set_cursor_return();
        } else {
            self.draw_char(char_to_display, self.cursor_pos.0, self.cursor_pos.1, color)?;
            self.move_cursor()?;
        }
        Ok(())
    }

    /// Moves the cursor position to the next location, handling line wrapping and screen bounds.
    ///
    /// The method increments the cursor's horizontal position by the width of a single character.
    /// If the cursor reaches the end of the line, it wraps to the beginning of the next line and
    /// increments the vertical position by the height of a character. If the cursor moves beyond
    /// the vertical bounds of the screen, it returns an error.
    ///
    /// # Errors
    ///
    /// Returns `Err(DisplayError::OutOfScreenBounds)` if moving the cursor vertically
    /// would exceed the screen's height.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the cursor successfully moves to the new position.
    ///
    fn move_cursor(&mut self) -> DisplayResult<()> {
        let mut next_cursor_pos = self.cursor_pos;
        next_cursor_pos.0 += self.font.get_char_size().0 as u16;
        if next_cursor_pos.0 > self.size.unwrap().0 - self.font.get_char_size().0 as u16 {
            next_cursor_pos.0 = 0;
            next_cursor_pos.1 += self.font.get_char_size().1 as u16;
            if next_cursor_pos.1 > self.size.unwrap().1 - self.font.get_char_size().1 as u16 {
                return Err(DisplayError::OutOfScreenBounds);
            }
        }
        self.cursor_pos = next_cursor_pos;
        Ok(())
    }

    /// Sets the font size for the current object.
    ///
    /// # Parameters
    /// - `font`: The new font size to be set. This should be a value of type `FontSize`.
    ///
    /// # Remarks
    /// This function updates the `font` property of the object to the specified value.
    /// The new font size will replace any previously set value.
    pub fn set_font(&mut self, font: FontSize) -> DisplayResult<()> {
        self.font = font;
        Ok(())
    }

    /// Moves the cursor position down by one line, taking into account the character height of the font.
    ///
    /// # Behavior
    /// - The `cursor_pos.1` (the vertical position) is incremented by the height of a single character, as defined by the font's character size.
    /// - If the new vertical position exceeds the display's height limit, an error is returned.
    ///
    /// # Returns
    /// - `Ok(())` if the cursor was successfully moved to the next line within screen bounds.
    /// - `Err(DisplayError::OutOfScreenBounds)` if the operation would move the cursor beyond the bottom boundary of the screen.
    ///
    /// # Errors
    /// - Returns `DisplayError::OutOfScreenBounds` if the cursor is moved beyond the screen's vertical size.
    ///
    /// # Notes
    /// - The display size is expected to be set (via `self.size.unwrap()`), and the result relies on the font's character size.
    /// - Cursor movement assumes that the font, display size, and cursor position are valid and properly managed.
    ///
    pub fn set_cursor_line_feed(&mut self) -> DisplayResult<()> {
        self.cursor_pos.1 += self.font.get_char_size().1 as u16;
        if self.cursor_pos.1 > self.size.unwrap().1 - self.font.get_char_size().1 as u16 {
            Err(DisplayError::OutOfScreenBounds)
        } else {
            Ok(())
        }
    }

    /// Sets the cursor to the starting position of the current line.
    ///
    /// This function resets the horizontal position of the cursor (`cursor_pos.0`) to 0,
    /// effectively moving the cursor to the beginning of the current line.
    ///
    /// # Note
    /// This function does not modify the vertical position (`cursor_pos.1`) of the cursor,
    /// nor does it affect any other part of the cursor state.
    pub fn set_cursor_return(&mut self) {
        self.cursor_pos.0 = 0;
    }

    /// Sets the cursor position on the display.
    ///
    /// # Parameters
    /// - `x`: The horizontal position of the cursor, specified as a `u16`.
    /// - `y`: The vertical position of the cursor, specified as a `u16`.
    ///
    /// # Returns
    /// - `Ok(())`: If the cursor position was successfully updated.
    /// - `Err(DisplayError)`: If an error occurred preventing the operation.
    ///
    /// # Errors
    /// - `DisplayError::DisplayDriverNotInitialized`: Returned if the display driver
    ///   has not been initialized before attempting to set the cursor position.
    /// - `DisplayError::OutOfScreenBounds`: Returned if the specified `x` or `y`
    ///   coordinates exceed the screen boundaries
    pub fn set_cursor_pos(&mut self, x: u16, y: u16) -> DisplayResult<()> {
        if !self.initialized {
            return Err(DisplayError::DisplayDriverNotInitialized);
        }

        if x < self.size.unwrap().0 && y < self.size.unwrap().1 {
            self.cursor_pos.0 = x;
            self.cursor_pos.1 = y;
            Ok(())
        } else {
            Err(DisplayError::OutOfScreenBounds)
        }
    }

    /// Sets the color of the display to the specified value.
    ///
    /// # Arguments
    ///
    /// * `color` - A value of the `Colors` enum representing the color to set.
    ///
    /// # Returns
    ///
    /// * `DisplayResult<()>` - Returns `Ok(())` if the color is successfully set.
    ///
    /// # Errors
    ///
    /// This function does not produce any errors in its current implementation.
    pub fn set_color(&mut self, color: Colors) -> DisplayResult<()> {
        self.color = color;
        Ok(())
    }
}
