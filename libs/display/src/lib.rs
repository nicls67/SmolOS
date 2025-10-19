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

        // Clean the buffer
        self.clear(background_color, 0)?;

        Ok(())
    }

    /// Attempts to acquire a lock on the display interface for a specific caller.
    ///
    /// # Parameters
    /// - `caller_id` (u32): The unique identifier for the caller attempting to acquire the lock.
    ///
    /// # Returns
    /// - `DisplayResult<()>`: Returns `Ok(())` if the lock operation is successful; otherwise, returns
    ///   an error of type `DisplayError::HalError` if the Hardware Abstraction Layer (HAL) encounters an issue.
    ///
    /// # Errors
    /// - Returns `DisplayError::HalError` if the HAL fails to lock the interface for the given `caller_id`.
    ///
    /// # Panics
    /// - This function will panic if `self.hal` is `None` or if `self.hal_id` is `None`, as both are
    ///   expected to be initialized before calling this method.
    pub fn lock(&mut self, caller_id: u32) -> DisplayResult<()> {
        self.hal
            .as_mut()
            .unwrap()
            .lock_interface(self.hal_id.unwrap(), caller_id)
            .map_err(DisplayError::HalError)
    }

    /// Unlocks a hardware interface for a calling entity.
    ///
    /// This method attempts to unlock the hardware interface by invoking the `unlock_interface`
    /// function on the underlying Hardware Abstraction Layer (HAL). It is expected to be called when
    /// a specific caller, identified by `caller_id`, needs access to the interface.
    ///
    /// # Arguments
    ///
    /// * `caller_id` - A `u32` identifier representing the caller requesting the unlock operation.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the interface is successfully unlocked.
    /// * `Err(DisplayError::HalError)` if there is an error while unlocking the interface via the HAL.
    ///
    /// # Panics
    ///
    /// This method will panic if:
    /// * The `hal` or `hal_id` fields of the object are `None`, indicating that the HAL is not properly initialized.
    ///
    pub fn unlock(&mut self, caller_id: u32) -> DisplayResult<()> {
        self.hal
            .as_mut()
            .unwrap()
            .unlock_interface(self.hal_id.unwrap(), caller_id)
            .map_err(DisplayError::HalError)
    }

    /// Clears the display with the specified color and resets the cursor position to the top-left corner.
    ///
    /// # Parameters
    ///
    /// - `color`: The `Colors` enum value representing the color to fill the display with.
    /// - `caller_id`: A `u32` value representing the identifier of the operation's caller.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the clear operation is successful.
    /// - `Err(DisplayError::DisplayDriverNotInitialized)` if the display driver is not initialized.
    /// - `Err(DisplayError::HalError)` if an error occurs while interacting with the hardware abstraction layer (HAL).
    ///
    /// # Behavior
    ///
    /// - If the display driver is initialized (`self.initialized` is `true`), this function sends a command to
    ///   the hardware abstraction layer to clear the foreground layer of the display with the specified `color`.
    /// - The cursor position is reset to `(0, 0)` after clearing the display.
    /// - If the display driver is not initialized (`self.initialized` is `false`), an error of type
    ///   `DisplayError::DisplayDriverNotInitialized` is returned.
    ///
    /// # Errors
    ///
    /// - `DisplayError::DisplayDriverNotInitialized`: Indicates that the display driver has not been initialized
    ///   before attempting to call this function.
    /// - `DisplayError::HalError`: Indicates that a failure occurred during interaction with the hardware abstraction
    ///   layer (HAL), which is likely related to the `interface_write` function or command execution.
    pub fn clear(&mut self, color: Colors, caller_id: u32) -> DisplayResult<()> {
        if self.initialized {
            self.hal
                .as_mut()
                .unwrap()
                .interface_write(
                    self.hal_id.unwrap(),
                    caller_id,
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

    /// Switches the frame buffer used by the display and updates the hardware with the new address.
    ///
    /// # Parameters
    /// - `caller_id` - The identifier for the caller requesting the frame buffer switch.
    ///
    /// # Returns
    /// Returns a `DisplayResult` indicating success (`Ok(())`) or a `DisplayError` in case of failure.
    ///
    /// # Behavior
    /// 1. Switches the internal frame buffer to the next available one via the `switch` method of `frame_buffer`.
    /// 2. Retrieves the new frame buffer address (`fb_addr`) and sends this address to the hardware abstraction layer (HAL)
    ///    through `interface_write`, instructing it to use the new buffer for the foreground layer of the display.
    ///
    /// # Errors
    /// If an error occurs during the communication with the HAL, a `DisplayError::HalError` is returned with the details.
    ///
    /// # Requirements
    /// - The `frame_buffer` must be initialized (`Some` value).
    /// - The `hal` and `hal_id` must also be initialized (`Some` values).
    ///
    pub fn switch_frame_buffer(&mut self, caller_id: u32) -> DisplayResult<()> {
        let fb_addr = self.frame_buffer.as_mut().unwrap().switch();

        self.hal
            .as_mut()
            .unwrap()
            .interface_write(
                self.hal_id.unwrap(),
                caller_id,
                InterfaceWriteActions::Lcd(LcdActions::SetFbAddress(LcdLayer::FOREGROUND, fb_addr)),
            )
            .map_err(DisplayError::HalError)?;

        Ok(())
    }

    /// Draws a string on the display at the specified position with the given color.
    ///
    /// This function renders a string on the display starting at coordinates `(x, y)`.
    /// If a color is provided, it uses the specified color; otherwise, it defaults to
    /// the object's current color configuration. The function also ensures the display
    /// is properly initialized and performs an authorization check based on the
    /// `caller_id`.
    ///
    /// # Parameters
    ///
    /// - `string`: A reference to the string to be drawn on the display.
    /// - `x`: The horizontal (x-axis) starting position for the string in pixels.
    /// - `y`: The vertical (y-axis) starting position for the string in pixels.
    /// - `color`: An optional `Colors` enum specifying the color of the string.
    ///     If `None`, the default color of the display is used.
    /// - `caller_id`: A unique identifier for the caller to perform authorization
    ///   checks for modifying the display.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the string is successfully drawn on the display.
    /// - `Err(DisplayError)` if an error occurs. Possible errors include:
    ///   - `DisplayError::DisplayDriverNotInitialized`: The display driver is not initialized.
    ///   - `DisplayError::HalError`: An error occurred in the hardware abstraction layer.
    ///
    /// # Behavior
    /// - The function calculates where each character of the string should be drawn,
    ///   accounting for font size and starting position.
    /// - The string is rendered character by character, adjusting the frame buffer's
    ///   address accordingly.
    /// - The `color` is converted to an ARGB format and applied to the characters.
    ///
    /// # Errors
    /// - If the display has not been initialized (`self.initialized == false`),
    ///   the function returns `DisplayError::DisplayDriverNotInitialized`.
    /// - If the `authorize_action` function of the hardware abstraction layer (HAL)
    ///   fails, the function returns `DisplayError::HalError`.
    ///
    /// # Important Notes
    /// - This function assumes the display is being updated through a frame buffer.
    /// - The `caller_id` must match the authorized ID for the display's HAL interface,
    ///   or the operation will fail.
    pub fn draw_string(
        &mut self,
        string: &str,
        x: u16,
        y: u16,
        color: Option<Colors>,
        caller_id: u32,
    ) -> DisplayResult<()> {
        // Returns error if not initialized
        if !self.initialized {
            return Err(DisplayError::DisplayDriverNotInitialized);
        }

        // Check for lock on interface
        self.authorize_display_action(caller_id)?;

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

    /// Draws a character on the display at the specified position with an optional color.
    ///
    /// # Parameters
    /// - `char_to_display`: The ASCII value of the character to be displayed.
    /// - `x`: The x-coordinate where the character should be drawn (in pixels).
    /// - `y`: The y-coordinate where the character should be drawn (in pixels).
    /// - `color`: An optional color parameter of type `Colors`. If this is `None`, the default color
    ///   of the display will be used.
    /// - `caller_id`: A unique identifier for the caller, used for authorization. This ensures
    ///   actions are performed by authorized entities.
    ///
    /// # Returns
    /// - `Ok(())` if the character is successfully drawn on the display.
    /// - `Err(DisplayError)` if the display is not initialized, authorization fails, or if there is
    ///   an error during drawing operations.
    ///
    /// # Errors
    /// - `DisplayError::DisplayDriverNotInitialized`: Returned if the display driver is not initialized.
    /// - `DisplayError::HalError`: Returned if authorization for the action fails or the underlying
    ///   hardware abstraction layer (HAL) encounters an error.
    /// - Other errors related to framebuffer or rendering may also be propagated.
    ///
    /// # Behavior
    /// - Verifies that the display has been initialized before performing any operations. If not, an
    ///   error is returned.
    /// - Checks for a lock on the hardware abstraction layer (HAL) and ensures that the caller is
    ///   authorized to perform the drawing action.
    /// - Retrieves the size of the character from the current font and computes the destination
    ///   address within the frame buffer for rendering.
    /// - Uses the provided color or falls back to the default display color if one is not specified.
    /// - Draws the character at the specified position on the display's frame buffer.
    ///
    /// # Notes
    /// - Ensure the display is properly initialized before invoking this method.
    /// - The method relies on the font and color information configured on the display.
    /// - The caller needs appropriate authorization access to perform the drawing operation.
    pub fn draw_char(
        &mut self,
        char_to_display: u8,
        x: u16,
        y: u16,
        color: Option<Colors>,
        caller_id: u32,
    ) -> DisplayResult<()> {
        // Returns error if not initialized
        if !self.initialized {
            return Err(DisplayError::DisplayDriverNotInitialized);
        }

        // Check for lock on interface
        self.authorize_display_action(caller_id)?;

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

    /// Draws a given string at the current cursor position on the display. Each character
    /// in the string is processed and rendered sequentially.
    ///
    /// # Parameters
    ///
    /// * `string` - A reference to the string slice (`&str`) that contains the characters to be drawn.
    /// * `color` - An optional parameter specifying the color to use for drawing the string.
    ///             If `None` is provided, a default color may be applied.
    /// * `caller_id` - An identifier (u32) of the caller, potentially used for tracking, logging,
    ///                 or access control.
    ///
    /// # Returns
    ///
    /// * `DisplayResult<()>` - A result type which is `Ok(())` if the string is
    pub fn draw_string_at_cursor(
        &mut self,
        string: &str,
        color: Option<Colors>,
        caller_id: u32,
    ) -> DisplayResult<()> {
        // Draw the string at the current cursor position
        for char_to_display in string.as_bytes() {
            self.draw_char_at_cursor(*char_to_display, color, caller_id)?;
        }
        Ok(())
    }

    ///
    /// Draws a character at the current cursor position on the display.
    ///
    /// # Parameters
    /// - `char_to_display`: A `u8` representing the ASCII value of the character to be displayed.
    /// - `color`: An `Option<Colors>` representing the optional color of the character. If `None`, a default color will be used.
    /// - `caller_id`: A `u32` identifier for the caller, used for tracking or logging purposes.
    ///
    /// # Behavior
    /// - If the character is a newline (`b'\n'`), it moves the cursor to the next line using `set_cursor_line_feed`.
    /// - If the character is a carriage return (`b'\r'`), it moves the cursor back to the start of the current line using `set_cursor_return`.
    /// - For any other character:
    ///   - The character is drawn at the current cursor position using the `draw_char` method.
    ///   - The cursor is then advanced to the next position using the `move_cursor` method.
    ///
    /// # Returns
    /// - A `DisplayResult<()>`, which indicates success or contains an error if an operation fails.
    ///
    /// # Errors
    /// - May return an error if:
    ///   - Moving the cursor fails.
    ///   - Drawing the character on the display fails.
    ///
    pub fn draw_char_at_cursor(
        &mut self,
        char_to_display: u8,
        color: Option<Colors>,
        caller_id: u32,
    ) -> DisplayResult<()> {
        if char_to_display == b'\n' {
            self.set_cursor_line_feed(caller_id)?;
        } else if char_to_display == b'\r' {
            self.set_cursor_return(caller_id)?;
        } else {
            self.draw_char(
                char_to_display,
                self.cursor_pos.0,
                self.cursor_pos.1,
                color,
                caller_id,
            )?;
            self.move_cursor(caller_id)?;
        }
        Ok(())
    }

    /// Moves the cursor position on the display, ensuring it remains within the allowable bounds.
    ///
    /// This function updates the cursor's position based on the width and height of the font's character size.
    /// If the cursor reaches the end of a line, it wraps to the beginning of the next line. If the cursor exceeds
    /// the display's allowable dimensions, an error (`DisplayError::OutOfScreenBounds`) is returned.
    ///
    /// Additionally, the function checks if the calling entity is authorized to perform this action through the
    /// provided hardware abstraction layer (HAL).
    ///
    /// # Parameters
    /// - `caller_id` (`u32`): The ID of the calling entity requesting the cursor move. This ID is used to authorize
    /// their action through the hardware abstraction layer (HAL).
    ///
    /// # Errors
    /// - `DisplayError::HalError`: Returned if the HAL denies the caller's authorization to perform the action.
    /// - `DisplayError::OutOfScreenBounds`: Returned if the cursor moves outside the allowable area of the display.
    ///
    /// # Returns
    /// - `DisplayResult<()>`: Returns `Ok(())` if the cursor was successfully moved, or an appropriate error otherwise.
    ///
    /// # Behavior
    /// - Checks for authorization by calling the `authorize_action` method of the HAL with the provided `caller_id`.
    /// - Moves the cursor horizontally by the font's character width.
    /// - If the next horizontal position exceeds the screen width, wraps the cursor to the beginning of the next line.
    /// - If the next line position exceeds the screen height, returns `DisplayError::OutOfScreenBounds`.
    ///
    fn move_cursor(&mut self, caller_id: u32) -> DisplayResult<()> {
        // Check for lock on interface
        self.authorize_display_action(caller_id)?;

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

    /// Sets the font size for the display.
    ///
    /// # Parameters
    /// - `font`: The `FontSize` to be applied to the display.
    /// - `caller_id`: A `u32` representing the identifier of the caller attempting to change the font.
    ///
    /// # Returns
    /// A `DisplayResult<()>` which:
    /// - Returns `Ok(())` if the font size is successfully updated.
    /// - Propagates any error that occurs during the authorization process.
    ///
    /// # Errors
    /// Returns an error if the `authorize_display_action` fails, indicating the caller is not authorized
    /// to perform this action.
    ///
    pub fn set_font(&mut self, font: FontSize, caller_id: u32) -> DisplayResult<()> {
        self.authorize_display_action(caller_id)?;

        self.font = font;
        Ok(())
    }

    /// Adjusts the cursor position by performing a line feed, moving the cursor down by the height
    /// of a character as defined by the font. Ensures the new position does not exceed the boundaries
    /// of the display.
    ///
    /// # Arguments
    ///
    /// * `caller_id` - The unique identifier of the caller attempting to perform this action. Used
    ///   for authorizing the display action.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the cursor position is updated successfully within the screen boundaries.
    /// * `Err(DisplayError::OutOfScreenBounds)` - If the new cursor position exceeds the screen's
    ///   bottom boundary.
    /// * `Err(DisplayError::AuthorizationFailed)` - If the caller is not authorized to perform the
    ///   action (propagated from `authorize_display_action`).
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// 1. The cursor's updated vertical position goes beyond the lower display boundary.
    /// 2. The caller is not authorized to modify the display (via `authorize_display_action`).
    ///
    /// # Behavior
    ///
    /// - The method first checks for authorization to ensure the caller has the right to modify
    ///   the display.
    /// - The cursor's vertical position (`cursor_pos.1`) is incremented by the font's character
    ///   height.
    /// - If the new vertical position exceeds the screen's height minus the character height, the
    ///   screen bounds are considered
    pub fn set_cursor_line_feed(&mut self, caller_id: u32) -> DisplayResult<()> {
        // Check for lock on interface
        self.authorize_display_action(caller_id)?;

        self.cursor_pos.1 += self.font.get_char_size().1 as u16;
        if self.cursor_pos.1 > self.size.unwrap().1 - self.font.get_char_size().1 as u16 {
            Err(DisplayError::OutOfScreenBounds)
        } else {
            Ok(())
        }
    }

    /// Sets the cursor position to the starting position (return to the beginning of the line).
    ///
    /// # Parameters
    /// - `caller_id` (`u32`): The identifier of the caller, used to authorize the action.
    ///
    /// # Returns
    /// `DisplayResult<()>`: Returns an `Ok(())` if the cursor position was successfully reset,
    /// or an error if the caller is not authorized to perform this action.
    ///
    /// # Errors
    /// - Returns an error if the `authorize_display_action` check fails, indicating the caller is not authorized.
    ///
    /// # Behavior
    /// - Resets the horizontal cursor position (`cursor_pos.0`) to `0`, effectively returning the cursor to the start.
    pub fn set_cursor_return(&mut self, caller_id: u32) -> DisplayResult<()> {
        // Check for lock on interface
        self.authorize_display_action(caller_id)?;

        self.cursor_pos.0 = 0;

        Ok(())
    }

    /// Sets the cursor position on the display to the specified coordinates.
    ///
    /// # Parameters
    /// - `x`: The x-coordinate of the position to set the cursor to. Must be within the screen bounds.
    /// - `y`: The y-coordinate of the position to set the cursor to. Must be within the screen bounds.
    /// - `caller_id`: An identifier for the caller, which is used to verify the authorization of the action.
    ///
    /// # Returns
    /// - `Ok(())` if the cursor position was successfully updated.
    /// - `Err(DisplayError::DisplayDriverNotInitialized)` if the display driver has not been initialized.
    /// - `Err(DisplayError::OutOfScreenBounds)` if the specified coordinates are outside the screen bounds.
    /// - An error propagated from the `authorize_display_action` method if the caller is not authorized.
    ///
    /// # Preconditions
    /// - The display driver must be initialized before calling this method.
    /// - The provided `x` and `y` coordinates must fall within the display's dimensions.
    ///
    /// # Behavior
    /// - If the display has not been initialized, the method immediately returns a `DisplayDriverNotInitialized` error.
    /// - The method checks the caller's authorization using `authorize_display_action`.
    /// - If the coordinates are valid and authorized, the internal `cursor_pos` is updated.
    /// - If the coordinates are outside the screen bounds, the method returns an `OutOfScreenBounds` error.
    pub fn set_cursor_pos(&mut self, x: u16, y: u16, caller_id: u32) -> DisplayResult<()> {
        if !self.initialized {
            return Err(DisplayError::DisplayDriverNotInitialized);
        }

        // Check for lock on interface
        self.authorize_display_action(caller_id)?;

        if x < self.size.unwrap().0 && y < self.size.unwrap().1 {
            self.cursor_pos.0 = x;
            self.cursor_pos.1 = y;
            Ok(())
        } else {
            Err(DisplayError::OutOfScreenBounds)
        }
    }

    /// Sets the display color to the specified value.
    ///
    /// # Parameters
    /// - `color`: The new color to set for the display. Must be a valid value of the `Colors` enum.
    /// - `caller_id`: The unique identifier of the caller attempting to set the color. It is used
    ///   for authorization purposes.
    ///
    /// # Returns
    /// - `DisplayResult<()>`: Returns `Ok(())` if the color is successfully set.
    ///   Returns an error if the caller is not authorized or another issue occurs.
    ///
    /// # Errors
    /// - Returns an error if the `caller_id` does not pass the authorization
    ///   check using `authorize_display_action`.
    ///
    /// # Notes
    /// - The method requires successful authorization before changing the display's color.
    pub fn set_color(&mut self, color: Colors, caller_id: u32) -> DisplayResult<()> {
        // Check for lock on interface
        self.authorize_display_action(caller_id)?;

        self.color = color;
        Ok(())
    }

    /// Attempts to authorize a display action for the given caller ID.
    ///
    /// # Parameters
    /// - `caller_id`: The unique identifier of the caller attempting to perform the display action.
    ///
    /// # Returns
    /// - `Ok(())`: If the authorization succeeds.
    /// - `Err(DisplayError::HalError)`: If an error occurs during the authorization process.
    ///
    /// # Behavior
    /// - This function interacts with the hardware abstraction layer (HAL) to determine
    ///   if the requested action can be authorized.
    /// - It first checks for the existence of the HAL instance (`self.hal`) and halts execution
    ///   if the instance is not present.
    /// - Once the HAL instance is confirmed, the action is authorized using the HAL's
    ///   `authorize_action` method, which takes the
    fn authorize_display_action(&mut self, caller_id: u32) -> DisplayResult<()> {
        // Check for lock on interface
        self.hal
            .as_mut()
            .unwrap()
            .authorize_action(self.hal_id.unwrap(), caller_id)
            .map_err(DisplayError::HalError)
    }
}
