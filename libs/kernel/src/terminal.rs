//! Terminal interface for the kernel.
//!
//! This module provides a small terminal abstraction backed by a [`ConsoleOutput`]
//! (typically a USART). The terminal has two primary modes:
//! - **Prompt mode**: user input is echoed, accumulated into a line buffer, and
//!   executed as an application command on carriage return (`'\r'`).
//! - **Display mode**: output formatting requests are rendered to the console;
//!   user input is ignored.
//!
//! A HAL callback (`terminal_prompt_callback`) is registered in prompt mode so
//! that incoming bytes are read from the interface and forwarded to
//! [`Terminal::process_input`].

use crate::KernelError::TerminalError;
use crate::KernelErrorLevel::Error;

use crate::console_output::{ConsoleFormatting, ConsoleOutput};
use crate::data::Kernel;
use crate::ident::KERNEL_MASTER_ID;
use crate::terminal::TerminalState::{Display, Prompt};
use crate::{KernelResult, SysCallHalActions, syscall_hal};

use display::Colors;
use hal_interface::{BUFFER_SIZE, InterfaceReadAction, InterfaceReadResult};
use heapless::{String, Vec, format};

#[derive(PartialEq, Clone, Copy, Debug)]
enum TerminalState {
    /// Terminal is stopped
    Stopped,
    /// Terminal is in prompt mode
    Prompt,
    /// Terminal is in display-only mode
    Display,
}

pub struct Terminal {
    output: ConsoleOutput,
    line_buffer: String<256>,
    mode: TerminalState,
    cursor_pos: usize,
    display_mirror: Option<ConsoleOutput>,
}

impl Terminal {
    /// Create a new terminal instance bound to a named USART console output.
    ///
    /// # Parameters
    /// - `name`: Static name/identifier used to select the USART interface.
    ///
    /// # Returns
    /// - `Ok(Terminal)` on success.
    /// - `Err(_)` if the underlying [`ConsoleOutput::new`] fails.
    ///
    /// # Errors
    /// Propagates any [`KernelError`](crate::KernelError) produced by the console
    /// output initialization (e.g., interface open/configuration failures).
    pub fn new(name: &'static str) -> KernelResult<Terminal> {
        let output = ConsoleOutput::new(
            crate::console_output::ConsoleOutputType::Usart(name),
            Colors::White,
        )?;

        Ok(Terminal {
            output,
            line_buffer: String::new(),
            mode: TerminalState::Stopped,
            cursor_pos: 0,
            display_mirror: None,
        })
    }

    /// Enable or disable mirroring of terminal output to the display.
    ///
    /// When enabled (`display_mirror == true`) and no mirror exists yet, this
    /// function will create a secondary [`ConsoleOutput`] targeting the display
    /// backend (`ConsoleOutputType::Display`) and store it in
    /// [`Terminal::display_mirror`].
    ///
    /// When disabled (`display_mirror == false`) and a mirror is currently
    /// active, this function will release the mirror output and clear the stored
    /// handle.
    ///
    /// # Parameters
    /// - `display_mirror`: `true` to enable mirroring, `false` to disable it.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    ///
    /// # Errors
    /// - Propagates any error produced by [`ConsoleOutput::new`] when enabling.
    /// - Propagates any error produced by [`ConsoleOutput::release`] when disabling.
    pub fn set_display_mirror(&mut self, display_mirror: bool) -> KernelResult<()> {
        if display_mirror && self.display_mirror.is_none() {
            self.display_mirror = Some(ConsoleOutput::new(
                crate::console_output::ConsoleOutputType::Display,
                Colors::White,
            )?);
        } else if let Some(mirror) = self.display_mirror.as_mut()
            && !display_mirror
        {
            mirror.release()?;
            self.display_mirror = None;
        }
        Ok(())
    }

    /// Switch the terminal to prompt mode and print the prompt (`>`).
    ///
    /// In prompt mode, the terminal registers [`terminal_prompt_callback`] as the
    /// HAL callback for the underlying interface. User input is echoed and
    /// accumulated into an internal line buffer until carriage return (`'\r'`)
    /// triggers execution via `Kernel::apps().start_app(...)`.
    ///
    /// # Parameters
    /// - `&mut self`: The terminal to configure.
    ///
    /// # Returns
    /// - `Ok(())` if the callback is configured and the prompt is displayed.
    /// - `Err(_)` if configuring the callback or writing to the console fails.
    ///
    /// # Errors
    /// - Propagates errors from [`syscall_hal`] when configuring the callback.
    /// - Propagates errors from console output operations (`new_line`, `write_char`).
    pub fn set_prompt_mode(&mut self) -> KernelResult<()> {
        // Configure callback for user prompt data
        syscall_hal(
            self.output.interface_id,
            SysCallHalActions::ConfigureCallback(terminal_prompt_callback),
            KERNEL_MASTER_ID,
        )?;

        // Set mode to prompt
        if self.mode != Prompt {
            self.mode = Prompt;
            self.cursor_pos = 0;
            self.output.new_line()?;
            self.output.write_char('>')?;
        }

        Ok(())
    }

    /// Switch the terminal to display-only mode.
    ///
    /// In display mode, the terminal will render output provided via [`Terminal::write`],
    /// and it will ignore user input processing.
    ///
    /// # Parameters
    /// - `&mut self`: The terminal to configure.
    ///
    /// # Returns
    /// - Always returns `Ok(())`.
    ///
    /// # Errors
    /// This function does not currently produce errors.
    pub fn set_display_mode(&mut self) -> KernelResult<()> {
        // Set mode to display
        if self.mode != Display {
            self.mode = Display;
        }

        Ok(())
    }

    /// Write formatted output to the terminal when in display mode.
    ///
    /// If the terminal is not in display mode, this call is a no-op.
    ///
    /// # Parameters
    /// - `format`: The formatting instruction to render to the underlying console.
    ///
    /// # Returns
    /// - `Ok(())` on success (or if no-op due to not being in display mode).
    /// - `Err(_)` if any underlying console write operation fails.
    ///
    /// # Errors
    /// Propagates errors from [`ConsoleOutput`] operations such as `write_str`,
    /// `write_char`, `new_line`, and `clear_terminal`.
    pub fn write(&self, format: &ConsoleFormatting) -> KernelResult<()> {
        if self.mode == Display {
            match format {
                ConsoleFormatting::StrNoFormatting(text) => self.output.write_str(text)?,
                ConsoleFormatting::StrNewLineAfter(text) => {
                    self.output.write_str(text)?;
                    self.output.new_line()?;
                }
                ConsoleFormatting::StrNewLineBefore(text) => {
                    self.output.new_line()?;
                    self.output.write_str(text)?;
                }
                ConsoleFormatting::StrNewLineBoth(text) => {
                    self.output.new_line()?;
                    self.output.write_str(text)?;
                    self.output.new_line()?;
                }
                ConsoleFormatting::Newline => self.output.new_line()?,
                ConsoleFormatting::Char(c) => self.output.write_char(*c)?,
                ConsoleFormatting::Clear => self.output.clear_terminal()?,
            }

            if let Some(mirror) = self.display_mirror.as_ref() {
                match format {
                    ConsoleFormatting::StrNoFormatting(text) => mirror.write_str(text)?,
                    ConsoleFormatting::StrNewLineAfter(text) => {
                        mirror.write_str(text)?;
                        mirror.new_line()?;
                    }
                    ConsoleFormatting::StrNewLineBefore(text) => {
                        mirror.new_line()?;
                        mirror.write_str(text)?;
                    }
                    ConsoleFormatting::StrNewLineBoth(text) => {
                        mirror.new_line()?;
                        mirror.write_str(text)?;
                        mirror.new_line()?;
                    }
                    ConsoleFormatting::Newline => mirror.new_line()?,
                    ConsoleFormatting::Char(c) => mirror.write_char(*c)?,
                    ConsoleFormatting::Clear => mirror.clear_terminal()?,
                }
            }
        }

        Ok(())
    }

    /// Set the current output color for the terminal.
    ///
    /// This updates the `current_color` of the primary [`ConsoleOutput`] used by
    /// the terminal. If a display mirror output is enabled, its color is updated
    /// as well so mirrored output remains consistent.
    ///
    /// # Parameters
    /// - `color`: The new [`Colors`] value to use for subsequent output.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    ///
    /// # Errors
    /// Propagates any error returned by the underlying console output when
    /// applying the color change.
    pub fn set_color(&mut self, color: Colors) -> KernelResult<()> {
        if let Some(mirror) = self.display_mirror.as_mut() {
            mirror.current_color = color;
        }
        Ok(())
    }

    /// Process input bytes received from the HAL interface in prompt mode.
    ///
    /// The first byte (`buffer[0]`) is treated as the received character.
    /// - If it is carriage return (`'\r'`), the current line is executed via
    ///   `Kernel::apps().start_app(&self.line_buffer)`, and the line buffer is cleared.
    /// - Otherwise, the character is echoed and appended to the line buffer.
    ///
    /// # Parameters
    /// - `buffer`: A buffer read from the interface (expects at least 1 byte).
    ///
    /// # Returns
    /// - `Ok(())` on success (or if no-op due to not being in prompt mode).
    /// - `Err(_)` if echoing/writing fails or the line buffer overflows.
    ///
    /// # Errors
    /// - Propagates errors from console output operations (echo/prompt/newline).
    /// - Returns `TerminalError(Error, "Line buffer overflow")` if the internal
    ///   `line_buffer` cannot accept more characters.
    ///
    /// # Panics
    /// This function will panic if `buffer` is empty, because it unconditionally
    /// indexes `buffer[0]`.
    pub fn process_input(&mut self, buffer: Vec<u8, BUFFER_SIZE>) -> KernelResult<()> {
        // If the terminal is in prompt mode
        if self.mode == Prompt {
            // If the received character is a return character, process the line
            if buffer[0] == '\r' as u8 {
                // Start the requested command
                match Kernel::apps().start_app(&self.line_buffer) {
                    Ok(_) => {}
                    Err(err) => self
                        .output
                        .write_str(format!(256;"\r\n{}",err.to_string()).unwrap().as_str())?,
                };

                // Empty the line buffer and go to a new line
                self.line_buffer.clear();
                self.cursor_pos = 0;
                self.output.new_line()?;
                self.output.write_char('>')?;
            } else {
                // Echo the received character
                self.output.write_char(buffer[0] as char)?;

                // Store it into the line buffer
                self.line_buffer
                    .push(buffer[0] as char)
                    .map_err(|_| TerminalError(Error, "Line buffer overflow"))?;
                self.cursor_pos += 1;
            }
        }

        Ok(())
    }
}

/// HAL callback invoked when prompt input is available for the terminal interface.
///
/// This callback reads a buffer from the HAL interface identified by `id` and
/// forwards it to the kernel terminal's [`Terminal::process_input`] handler.
///
/// # Parameters
/// - `id`: Interface identifier (as provided by the HAL) that should be read.
///
/// # Returns
/// - This function returns `()` (FFI callback).
///
/// # Errors
/// This function does not return errors directly. Any error from [`syscall_hal`]
/// or [`Terminal::process_input`] is forwarded to `Kernel::errors().error_handler(&e)`.
pub extern "C" fn terminal_prompt_callback(id: u8) {
    let mut result = InterfaceReadResult::BufferRead(Vec::new());
    match syscall_hal(
        id as usize,
        SysCallHalActions::Read(InterfaceReadAction::BufferRead, &mut result),
        KERNEL_MASTER_ID,
    ) {
        Ok(()) => {
            if let InterfaceReadResult::BufferRead(buffer) = result {
                match Kernel::terminal().process_input(buffer) {
                    Ok(_) => {}
                    Err(e) => Kernel::errors().error_handler(&e),
                }
            }
        }
        Err(e) => Kernel::errors().error_handler(&e),
    }
}
