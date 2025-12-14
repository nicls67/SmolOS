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

/// Display driver abstraction wrapping an LCD HAL interface.
///
/// This type manages:
/// - An LCD HAL interface identifier and lock ownership (`kernel_master_id`)
/// - Screen size discovery
/// - A double frame buffer (via [`FrameBuffer`])
/// - Text rendering using the selected [`FontSize`]
/// - A text cursor and default text color
pub struct Display {
    hal_id: Option<usize>,
    kernel_master_id: u32,
    hal: Option<&'static mut Hal>,
    size: Option<(u16, u16)>,
    frame_buffer: Option<FrameBuffer>,
    initialized: bool,
    cursor_pos: (u16, u16),
    font: FontSize,
    color: Colors,
}

impl Display {
    /// Creates a new, non-initialized [`Display`] instance.
    ///
    /// The display must be initialized with [`Display::init`] before any drawing
    /// operations can succeed.
    ///
    /// # Parameters
    /// - `kernel_master_id`: The master/owner identifier used when locking the HAL
    ///   interface and issuing privileged LCD operations.
    ///
    /// # Returns
    /// A [`Display`] instance in a non-initialized state with:
    /// - cursor at `(0, 0)`
    /// - font set to [`FontSize::Font16`]
    /// - color set to [`Colors::White`]
    ///
    /// # Errors
    /// This function does not return errors.
    pub fn new(kernel_master_id: u32) -> Self {
        Self {
            hal_id: None,
            hal: None,
            kernel_master_id,
            size: None,
            frame_buffer: None,
            initialized: false,
            cursor_pos: (0, 0),
            font: Font16,
            color: Colors::White,
        }
    }

    /// Initializes the display driver and clears the screen.
    ///
    /// This function:
    /// 1. Resolves the LCD interface by name.
    /// 2. Enables the LCD.
    /// 3. Reads and stores the LCD size.
    /// 4. Stores the HAL reference and initializes the internal [`FrameBuffer`].
    /// 5. Locks the interface using `kernel_master_id`.
    /// 6. Clears the display to `background_color`.
    ///
    /// # Parameters
    /// - `lcd_name`: Name of the LCD interface as known by the HAL.
    /// - `hal`: A mutable static reference to the HAL implementation.
    /// - `background_color`: Color used to clear the display after initialization.
    ///
    /// # Returns
    /// - `Ok(())` if initialization succeeds.
    ///
    /// # Errors
    /// - [`DisplayError::HalError`] if HAL operations fail (lookup, enable, size read, lock, clear).
    /// - Any error returned by [`Display::clear`] (propagated), such as
    ///   [`DisplayError::DisplayDriverNotInitialized`] (should not occur if init flow succeeds).
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
            0,
            InterfaceWriteActions::Lcd(LcdActions::Enable(true)),
        )
        .map_err(DisplayError::HalError)?;

        // Get screen size
        self.size = match hal
            .interface_read(
                self.hal_id.unwrap(),
                0,
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

        // Try to lock the interface
        self.hal
            .as_mut()
            .unwrap()
            .lock_interface(self.hal_id.unwrap(), self.kernel_master_id)
            .map_err(DisplayError::HalError)?;

        // Clean the buffer
        self.clear(background_color)?;

        Ok(())
    }

    /// Clears the display and resets the cursor to `(0, 0)`.
    ///
    /// # Parameters
    /// - `color`: Background color used to clear the foreground layer.
    ///
    /// # Returns
    /// - `Ok(())` if the display was cleared successfully.
    ///
    /// # Errors
    /// - [`DisplayError::DisplayDriverNotInitialized`] if called before [`Display::init`].
    /// - [`DisplayError::HalError`] if the underlying HAL write fails.
    pub fn clear(&mut self, color: Colors) -> DisplayResult<()> {
        if self.initialized {
            self.hal
                .as_mut()
                .unwrap()
                .interface_write(
                    self.hal_id.unwrap(),
                    self.kernel_master_id,
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

    /// Switches the internal frame buffer and updates the LCD to display the new buffer.
    ///
    /// This uses the driver's [`FrameBuffer`] to flip buffers and then issues an LCD
    /// command to set the framebuffer base address.
    ///
    /// # Returns
    /// - `Ok(())` if the framebuffer address was successfully updated.
    ///
    /// # Errors
    /// - [`DisplayError::DisplayDriverNotInitialized`] if called before [`Display::init`].
    /// - [`DisplayError::HalError`] if the underlying HAL write fails.
    pub fn switch_frame_buffer(&mut self) -> DisplayResult<()> {
        // Returns error if not initialized
        if !self.initialized {
            return Err(DisplayError::DisplayDriverNotInitialized);
        }

        let fb_addr = self.frame_buffer.as_mut().unwrap().switch();

        self.hal
            .as_mut()
            .unwrap()
            .interface_write(
                self.hal_id.unwrap(),
                self.kernel_master_id,
                InterfaceWriteActions::Lcd(LcdActions::SetFbAddress(LcdLayer::FOREGROUND, fb_addr)),
            )
            .map_err(DisplayError::HalError)?;

        Ok(())
    }

    /// Draws an ASCII string at the provided pixel coordinates into the current frame buffer.
    ///
    /// Each character is rendered using the current [`FontSize`]. The provided `x`/`y`
    /// refer to the top-left pixel of the first character.
    ///
    /// # Parameters
    /// - `string`: UTF-8 string whose bytes are interpreted as ASCII codes.
    ///   Characters outside the supported ASCII range cause an error.
    /// - `x`: X coordinate in pixels of the first character.
    /// - `y`: Y coordinate in pixels of the first character.
    /// - `color`: Optional override color. If `None`, the current default color
    ///   set by [`Display::set_color`] is used.
    ///
    /// # Returns
    /// - `Ok(())` if all characters were drawn successfully.
    ///
    /// # Errors
    /// - [`DisplayError::DisplayDriverNotInitialized`] if called before [`Display::init`].
    /// - [`DisplayError::UnknownCharacter`] if any byte in `string` is outside
    ///   `FIRST_ASCII_CHAR..=LAST_ASCII_CHAR`.
    /// - Any error propagated from internal drawing routines.
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

    /// Draws a single ASCII character at the provided pixel coordinates into the current frame buffer.
    ///
    /// # Parameters
    /// - `char_to_display`: ASCII byte to render.
    /// - `x`: X coordinate in pixels of the character's top-left corner.
    /// - `y`: Y coordinate in pixels of the character's top-left corner.
    /// - `color`: Optional override color. If `None`, the current default color
    ///   set by [`Display::set_color`] is used.
    ///
    /// # Returns
    /// - `Ok(())` if the character was drawn successfully.
    ///
    /// # Errors
    /// - [`DisplayError::DisplayDriverNotInitialized`] if called before [`Display::init`].
    /// - [`DisplayError::UnknownCharacter`] if `char_to_display` is outside
    ///   `FIRST_ASCII_CHAR..=LAST_ASCII_CHAR`.
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

    /// Renders a single ASCII character glyph directly into the frame buffer memory.
    ///
    /// This is an internal routine used by [`Display::draw_char`] and [`Display::draw_string`].
    ///
    /// # Parameters
    /// - `char_to_display`: ASCII byte to render.
    /// - `fb_write_address`: Base address (in bytes) of the top-left pixel of the character
    ///   within the currently displayed frame buffer. The routine writes 32-bit ARGB pixels.
    /// - `char_size`: `(width, height)` in pixels for the current font glyph.
    /// - `color_argb`: Pixel color written for "set" glyph pixels, encoded as ARGB `u32`.
    ///   Unset pixels are written as `0`.
    ///
    /// # Returns
    /// - `Ok(())` if the glyph was written successfully.
    ///
    /// # Errors
    /// - [`DisplayError::UnknownCharacter`] if `char_to_display` is outside
    ///   `FIRST_ASCII_CHAR..=LAST_ASCII_CHAR`.
    ///
    /// # Safety
    /// This function performs raw pointer writes into the frame buffer memory.
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
                    } else {
                        unsafe {
                            *(fb_write_address as *mut u32) = 0;
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

    /// Draws a string starting at the current cursor position.
    ///
    /// For each byte in `string`:
    /// - `\n` advances the cursor to the next line (line feed).
    /// - `\r` returns the cursor to the start of the current line (carriage return).
    /// - Any other byte is drawn as an ASCII glyph at the cursor and the cursor is advanced.
    ///
    /// # Parameters
    /// - `string`: UTF-8 string whose bytes are interpreted as ASCII codes.
    /// - `color`: Optional override color for all characters. If `None`, the current
    ///   default color is used.
    ///
    /// # Returns
    /// - `Ok(())` if the entire string was processed successfully.
    ///
    /// # Errors
    /// - [`DisplayError::DisplayDriverNotInitialized`] if called before [`Display::init`].
    /// - [`DisplayError::UnknownCharacter`] if any non-control byte is outside the supported
    ///   ASCII range.
    /// - [`DisplayError::OutOfScreenBounds`] if advancing the cursor moves past the bottom
    ///   of the screen.
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

    /// Draws a single character at the current cursor position and updates the cursor.
    ///
    /// Control characters:
    /// - `\n`: performs a line feed (moves cursor down by one character height).
    /// - `\r`: performs a carriage return (sets cursor X to 0).
    ///
    /// Otherwise, the character is drawn and the cursor advances by one character width,
    /// wrapping to the next line if necessary.
    ///
    /// # Parameters
    /// - `char_to_display`: The byte to process as either a control character (`\n`, `\r`)
    ///   or an ASCII glyph.
    /// - `color`: Optional override color. If `None`, the current default color is used.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    ///
    /// # Errors
    /// - [`DisplayError::DisplayDriverNotInitialized`] if called before [`Display::init`].
    /// - [`DisplayError::UnknownCharacter`] if a non-control byte is outside the supported range.
    /// - [`DisplayError::OutOfScreenBounds`] if cursor movement would exceed screen bounds.
    pub fn draw_char_at_cursor(
        &mut self,
        char_to_display: u8,
        color: Option<Colors>,
    ) -> DisplayResult<()> {
        if char_to_display == b'\n' {
            self.set_cursor_line_feed()?;
        } else if char_to_display == b'\r' {
            self.set_cursor_return()?;
        } else {
            self.draw_char(char_to_display, self.cursor_pos.0, self.cursor_pos.1, color)?;
            self.move_cursor()?;
        }
        Ok(())
    }

    /// Advances the cursor by one character cell, with line wrapping.
    ///
    /// Cursor advancement rules:
    /// - Increments X by the current font width.
    /// - If X would exceed the last full character cell of the line, wraps X to `0`
    ///   and increments Y by the current font height.
    ///
    /// # Returns
    /// - `Ok(())` if the cursor moved successfully.
    ///
    /// # Errors
    /// - [`DisplayError::DisplayDriverNotInitialized`] if called before [`Display::init`].
    /// - [`DisplayError::OutOfScreenBounds`] if moving would exceed the bottom of the screen.
    fn move_cursor(&mut self) -> DisplayResult<()> {
        if !self.initialized {
            return Err(DisplayError::DisplayDriverNotInitialized);
        }

        // Move cursor
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

    /// Sets the active font used for subsequent text rendering.
    ///
    /// # Parameters
    /// - `font`: Font size to use for subsequent draw operations.
    ///
    /// # Returns
    /// - `Ok(())` always.
    ///
    /// # Errors
    /// This function does not currently return errors.
    pub fn set_font(&mut self, font: FontSize) -> DisplayResult<()> {
        self.font = font;
        Ok(())
    }

    /// Moves the cursor down by one character height (line feed).
    ///
    /// # Returns
    /// - `Ok(())` if the cursor remains within bounds.
    ///
    /// # Errors
    /// - [`DisplayError::DisplayDriverNotInitialized`] if called before [`Display::init`].
    /// - [`DisplayError::OutOfScreenBounds`] if the new cursor Y would exceed the screen height.
    fn set_cursor_line_feed(&mut self) -> DisplayResult<()> {
        if !self.initialized {
            return Err(DisplayError::DisplayDriverNotInitialized);
        }

        self.cursor_pos.1 += self.font.get_char_size().1 as u16;
        if self.cursor_pos.1 > self.size.unwrap().1 - self.font.get_char_size().1 as u16 {
            Err(DisplayError::OutOfScreenBounds)
        } else {
            Ok(())
        }
    }

    /// Sets the cursor X position to the start of the current line (carriage return).
    ///
    /// # Returns
    /// - `Ok(())` always.
    ///
    /// # Errors
    /// This function does not currently return errors.
    fn set_cursor_return(&mut self) -> DisplayResult<()> {
        self.cursor_pos.0 = 0;
        Ok(())
    }

    /// Sets the cursor position in pixels.
    ///
    /// # Parameters
    /// - `x`: X coordinate in pixels.
    /// - `y`: Y coordinate in pixels.
    ///
    /// # Returns
    /// - `Ok(())` if `x` and `y` are within screen bounds.
    ///
    /// # Errors
    /// - [`DisplayError::DisplayDriverNotInitialized`] if called before [`Display::init`].
    /// - [`DisplayError::OutOfScreenBounds`] if `x` or `y` lies outside the screen size.
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

    /// Sets the default color used by drawing operations when `color: None` is provided.
    ///
    /// # Parameters
    /// - `color`: New default drawing color.
    ///
    /// # Returns
    /// - `Ok(())` always.
    ///
    /// # Errors
    /// This function does not currently return errors.
    pub fn set_color(&mut self, color: Colors) -> DisplayResult<()> {
        self.color = color;
        Ok(())
    }
}
