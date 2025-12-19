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
    app_exe_in_progress: Option<u32>,
}

impl Terminal {
    /// Construct a new [`Terminal`] bound to a named USART console output.
    ///
    /// This initializes the primary [`ConsoleOutput`] as a USART backend using
    /// the provided `name` and a default color of [`Colors::White`]. The terminal
    /// starts in the [`TerminalState::Stopped`] state with an empty line buffer,
    /// cursor position at `0`, and no display mirror configured.
    ///
    /// # Parameters
    /// - `name`: Static name/identifier used by the HAL to select the USART interface.
    ///
    /// # Returns
    /// - `Ok(Terminal)` on success.
    /// - `Err(_)` if creating the underlying [`ConsoleOutput`] fails.
    pub fn new(name: &'static str) -> KernelResult<Terminal> {
        Ok(Terminal {
            output: ConsoleOutput::new(
                crate::console_output::ConsoleOutputType::Usart(name),
                Colors::White,
            ),
            line_buffer: String::new(),
            mode: TerminalState::Stopped,
            cursor_pos: 0,
            display_mirror: None,
            app_exe_in_progress: None,
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
            ));
            self.display_mirror.as_mut().unwrap().initialize()?;
        } else if let Some(mirror) = self.display_mirror.as_mut()
            && !display_mirror
        {
            mirror.release()?;
            self.display_mirror = None;
        }
        Ok(())
    }

    /// Switch the terminal into prompt mode.
    ///
    /// Prompt mode enables interactive input:
    /// - Ensures the underlying output interface is initialized.
    /// - Registers the HAL callback [`terminal_prompt_callback`] so incoming bytes
    ///   are forwarded to [`Terminal::process_input`].
    /// - If transitioning from another mode, resets the cursor state and prints a
    ///   new prompt (`>`).
    ///
    /// # Returns
    /// - `Ok(())` on success.
    ///
    /// # Errors
    /// Propagates errors from initializing the underlying [`ConsoleOutput`] or from
    /// configuring the HAL callback via [`syscall_hal`].
    pub fn set_prompt_mode(&mut self) -> KernelResult<()> {
        // Initialize output interface if not already initialized
        if self.output.interface_id.is_none() {
            self.output.initialize()?;
        }

        // Configure callback for user prompt data
        syscall_hal(
            self.output.interface_id.unwrap(),
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

    /// Switch the terminal into display mode.
    ///
    /// Display mode is intended for output-only operation:
    /// - Ensures the underlying output interface is initialized.
    /// - Sets the terminal state to [`TerminalState::Display`].
    ///
    /// While in display mode, [`Terminal::write`] will render output to the
    /// console (and optionally to the configured display mirror), and user input
    /// will be ignored by [`Terminal::process_input`].
    ///
    /// # Returns
    /// - `Ok(())` on success.
    ///
    /// # Errors
    /// Propagates errors from initializing the underlying [`ConsoleOutput`].
    pub fn set_display_mode(&mut self) -> KernelResult<()> {
        // Initialize output interface if not already initialized
        if self.output.interface_id.is_none() {
            self.output.initialize()?;
        }

        // Set mode to display
        if self.mode != Display {
            self.mode = Display;
        }

        Ok(())
    }

    /// Write formatted output to the terminal (and optionally to the display mirror).
    ///
    /// This method renders the provided [`ConsoleFormatting`] to the terminal's
    /// primary [`ConsoleOutput`]. If a display mirror has been enabled via
    /// [`Terminal::set_display_mirror`], the same formatting operation is also
    /// applied to the mirror output.
    ///
    /// # Parameters
    /// - `format`: The [`ConsoleFormatting`] variant describing what to render.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    ///
    /// # Errors
    /// Propagates any error returned by the underlying [`ConsoleOutput`] methods
    /// (e.g., `write_str`, `write_char`, `new_line`, or `clear_terminal`) for either
    /// the primary output or the optional mirror output.
    pub fn write(&self, format: &ConsoleFormatting) -> KernelResult<()> {
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

    /// Process a buffer of input bytes received from the terminal interface.
    ///
    /// In [`TerminalState::Prompt`] mode, this function implements a simple line
    /// editor:
    /// - Non-`'\r'` bytes are echoed to the terminal and appended to the internal
    ///   line buffer.
    /// - On carriage return (`'\r'`), the accumulated line is treated as an
    ///   application command and is started via [`Kernel::apps().start_app`]. If
    ///   the application starts successfully, the terminal device is locked to
    ///   that application.
    ///
    /// In other terminal modes, the input is ignored.
    ///
    /// # Parameters
    /// - `buffer`: A byte buffer read from the HAL interface (typically containing
    ///   one byte for prompt input).
    ///
    /// # Returns
    /// - `Ok(())` on success.
    ///
    /// # Errors
    /// - Returns a terminal error if the internal line buffer overflows.
    /// - Propagates any I/O error from writing to the underlying console output.
    /// - Propagates any error from locking the terminal device after starting an app.
    pub fn process_input(&mut self, buffer: Vec<u8, BUFFER_SIZE>) -> KernelResult<()> {
        // If the terminal is in prompt mode
        if self.mode == Prompt {
            // If the received character is a return character, process the line
            if buffer[0] == '\r' as u8 {
                // Start the requested command
                match Kernel::apps().start_app(&self.line_buffer) {
                    Ok(app_id) => {
                        self.app_exe_in_progress = Some(app_id);
                        // Lock terminal for this app
                        Kernel::devices().lock(crate::DeviceType::Terminal, app_id)?;
                    }
                    Err(err) => {
                        self.output
                            .write_str(format!(256;"\r\n{}",err.to_string()).unwrap().as_str())?;
                        self.cursor_pos = 0;
                        self.output.new_line()?;
                        self.output.write_char('>')?;
                    }
                };
                self.line_buffer.clear();
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

    pub fn app_exit_notifier(&mut self, app_exit_id: u32) -> KernelResult<()> {
        if let Some(id) = self.app_exe_in_progress {
            if id == app_exit_id {
                self.app_exe_in_progress = None;
                Kernel::devices().unlock(crate::DeviceType::Terminal, id)?;
                self.cursor_pos = 0;
                self.output.new_line()?;
                self.output.write_char('>')?;
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
