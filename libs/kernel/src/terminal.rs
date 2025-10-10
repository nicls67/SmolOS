use crate::data::Kernel as KernelData;
use crate::terminal::TerminalState::Kernel;
use crate::{KernelError, KernelResult};
use display::Colors;
use hal_interface::{InterfaceWriteActions, UartWriteActions};
use heapless::{String, Vec};

/// Represents different kinds of terminal text formatting or operations.
///
/// This enum is used to specify how text or characters should be written to
/// the terminal, as well as other terminal-specific operations such as clearing
/// the screen.
///
/// # Variants
///
/// * `StrNoFormatting(&'a str)`
///     - Writes the given string to the terminal without any additional formatting or newlines.
/// * `StrNewLineAfter(&'a str)`
///     - Writes the given string to the terminal and appends a new line afterwards.
/// * `StrNewLineBefore(&'a str)`
///     - Writes the given string to the terminal after inserting a new line.
/// * `StrNewLineBoth(&'a str)`
///     - Writes the given string to the terminal preceded and followed by a new line.
/// * `Newline`
///     - Adds a new line without writing any additional content.
/// * `Char(char)`
///     - Writes a single character to the terminal.
/// * `Clear`
///     - Clears the terminal screen.
///
pub enum TerminalFormatting<'a> {
    /// No formatting is done
    StrNoFormatting(&'a str),
    /// New line is added after write
    StrNewLineAfter(&'a str),
    /// New line is added before write
    StrNewLineBefore(&'a str),
    /// New lines are added before and after write
    StrNewLineBoth(&'a str),
    /// Only adds a new line
    Newline,
    /// Writes a single character
    Char(char),
    /// Clears the terminal
    Clear,
}

#[derive(Copy, Clone)]
pub enum TerminalType {
    Usart(&'static str),
    Display,
}

/// Represents the state of a terminal.
///
/// The `TerminalState` enum defines various operational states
/// that a terminal can be in, providing control over its interaction
/// with users and kernel processes.
#[derive(PartialEq)]
enum TerminalState {
    /// Terminal is stopped
    Stopped,
    /// Terminal is started, waiting for user input, kernel writes are disabled
    User,
    /// Terminal is started, open to kernel writes
    Kernel,
}

/// Escape sequence
#[derive(PartialEq)]
enum EscapeSeqState {
    NotInEcsSeq,
    FirstRcv,
    SecRcv,
    ThirdRcv,
}

const MAX_TERMINALS: usize = 8;

pub struct Terminal {
    interface_id: Vec<usize, MAX_TERMINALS>,
    terminals: Vec<TerminalType, MAX_TERMINALS>,
    line_buffer: String<256>,
    state: TerminalState,
    escape_seq: EscapeSeqState,
    cursor_pos: usize,
    current_color: Colors,
}

impl Terminal {
    /// Creates a new `Terminal` instance with the given terminals vector.
    ///
    /// # Parameters
    /// - `terminals`: A fixed-size `Vec` containing terminal types, used to initialize the terminal
    ///   context. This vector can hold up to 8 terminal types.
    ///
    /// # Returns
    /// A new instance of the `Terminal` struct configured with default settings and the provided
    /// terminal types.
    ///
    /// # Default Values
    /// - `interface_id`: Initialized with a vector containing `MAX_TERMINALS` zeroes.
    /// - `line_buffer`: Starts as an empty string.
    /// - `state`: Set to `TerminalState::Stopped`.
    /// - `escape_seq`: Defaults to `EscapeSeqState::NotInEcsSeq`.
    /// - `cursor_pos`: Initialized to `0`.
    /// - `current_color`: Set to `Colors::White`.
    ///
    /// # Panics
    /// This function will panic if the creation of `interface_id` vector fails, which is unlikely
    /// unless system memory constraints are exceeded.
    ///
    pub fn new(terminals: Vec<TerminalType, 8>) -> Terminal {
        Terminal {
            interface_id: Vec::from_slice(&[0; MAX_TERMINALS]).unwrap(),
            terminals,
            line_buffer: String::new(),
            state: TerminalState::Stopped,
            escape_seq: EscapeSeqState::NotInEcsSeq,
            cursor_pos: 0,
            current_color: Colors::White,
        }
    }

    pub fn start(&mut self) -> KernelResult<()> {
        if self.state != TerminalState::User {
            self.set_user_mode()?;
        }
        Ok(())
    }

    pub fn set_user_mode(&mut self) -> KernelResult<()> {
        if self.state != TerminalState::User {
            self.state = TerminalState::User;
            self.escape_seq = EscapeSeqState::NotInEcsSeq;
            self.cursor_pos = 0;
            self.new_line()?;
            self.write_char('>')?;
        }
        Ok(())
    }

    pub fn set_kernel_state(&mut self) -> KernelResult<()> {
        // Retrieve interface id for all terminals
        for (i, terminal) in self.terminals.iter().enumerate() {
            if let TerminalType::Usart(name) = terminal {
                self.interface_id[i] = KernelData::hal()
                    .get_interface_id(name)
                    .map_err(KernelError::HalError)?;
            }
        }

        // Set state to kernel
        if self.state != Kernel {
            self.state = Kernel
        }
        Ok(())
    }

    /// Sets the current color of the object.
    ///
    /// # Arguments
    ///
    /// * `color` - A value of a type `display::Colors` that specifies the color
    ///   to set for the object.
    ///
    /// This method updates the `current_color` property of the object
    /// to the specified `color`.
    #[inline(always)]
    pub fn set_color(&mut self, color: display::Colors) {
        self.current_color = color;
    }

    /// Outputs a new line character sequence.
    ///
    /// This method writes the carriage return (`'\r'`) followed
    /// by the newline character (`'\n'`) to represent a new line.
    ///
    /// # Returns
    ///
    /// * `KernelResult<()>` - Returns `Ok(())` if both characters are
    /// successfully written. Returns an error if the write operation fails.
    ///
    /// # Errors
    ///
    /// This function propagates any errors encountered during the `write_char`
    /// operations for either the `'\r'` or `'\n'` character.
    ///
    /// # Inline
    ///
    /// This function is marked as `#[inline(always)]` to suggest
    /// the compiler always inlines it for performance reasons, as it
    /// is a frequently executed short function.
    ///
    #[inline(always)]
    fn new_line(&self) -> KernelResult<()> {
        self.write_char('\r')?;
        self.write_char('\n')
    }

    /// Writes formatted output to the terminal based on the specified formatting style.
    ///
    /// # Parameters
    /// - `format`: A reference to a `TerminalFormatting` enum that specifies the formatting
    ///   details for the output. The possible variants of `TerminalFormatting` are:
    ///     - `StrNoFormatting(text)`: Writes the provided `text` without any additional formatting.
    ///     - `StrNewLineAfter(text)`: Writes the provided `text` and appends a new line after it.
    ///     - `StrNewLineBefore(text)`: Writes a new line before the provided `text` and then writes the text.
    ///     - `StrNewLineBoth(text)`: Writes a new line before the provided `text`, writes the text,
    ///       and then adds a new line after it.
    ///     - `Newline`: Writes a single new line to the terminal.
    ///     - `Char(c)`: Writes the specified character `c` to the terminal.
    ///     - `Clear`: Clears the entire terminal display.
    ///
    /// # Returns
    /// - `KernelResult<()>`: Returns `Ok(())` if the operation is successful; otherwise, returns
    ///   an error wrapped in a `KernelResult`.
    ///
    /// # Conditions
    /// - This function executes only if the `self.state` is `Kernel`. If the state is not `Kernel`,
    ///   the function does nothing.
    ///
    /// # Errors
    /// - Propagates any errors that occur during writing actions, such as `write_str`, `write_char`,
    ///   `new_line`, or `clear_terminal`.
    ///
    pub fn write(&self, format: &TerminalFormatting) -> KernelResult<()> {
        if self.state == Kernel {
            match format {
                TerminalFormatting::StrNoFormatting(text) => self.write_str(text)?,
                TerminalFormatting::StrNewLineAfter(text) => {
                    self.write_str(text)?;
                    self.new_line()?;
                }
                TerminalFormatting::StrNewLineBefore(text) => {
                    self.new_line()?;
                    self.write_str(text)?;
                }
                TerminalFormatting::StrNewLineBoth(text) => {
                    self.new_line()?;
                    self.write_str(text)?;
                    self.new_line()?;
                }
                TerminalFormatting::Newline => self.new_line()?,
                TerminalFormatting::Char(c) => self.write_char(*c)?,
                TerminalFormatting::Clear => self.clear_terminal()?,
            }
        }
        Ok(())
    }

    /// Writes a single character to all configured terminal outputs.
    ///
    /// This method iterates through an internal list of terminal types (e.g., USART, Display)
    /// and writes the provided character to each terminal using their respective operations.
    /// Each terminal type has its own implementation for handling the character.
    ///
    /// # Arguments
    ///
    /// * `data` - The character to be written to the terminals.
    ///
    /// # Return
    ///
    /// * `KernelResult<()>` - Returns `Ok(())` if the character is successfully written to all terminals.
    ///   Returns an error wrapped in `KernelResult` if any operation fails while writing.
    ///
    /// # Behavior
    ///
    /// - For a terminal of type `TerminalType::Usart`, the character is converted to a `u8` and sent
    ///   via the UART interface using `KernelData::hal().interface_write()`.
    /// - For a terminal of type `TerminalType::Display`, the character is drawn on the display at the
    ///   current cursor position using `KernelData::display().draw_string_at_cursor()`.
    ///
    /// # Errors
    ///
    /// Returns the following errors if an operation fails:
    /// - `KernelError::HalError` if there is an issue writing to a `TerminalType::Usart`.
    /// - `KernelError::DisplayError` if there is an issue writing/drawing to a `TerminalType::Display`.
    ///
    /// # Notes
    ///
    /// - The character is handled by their respective terminal type implementations based on the
    ///   `TerminalType` enum.
    /// - Assumes data is valid UTF-8, as it converts the `char` to a `&str` when writing to a display.
    ///   Errors that may arise are propagated using the `?` operator.
    ///
    fn write_char(&self, data: char) -> KernelResult<()> {
        for (i, terminal) in self.terminals.iter().enumerate() {
            match terminal {
                TerminalType::Usart(_) => KernelData::hal()
                    .interface_write(
                        self.interface_id[i],
                        InterfaceWriteActions::UartWrite(UartWriteActions::SendChar(data as u8)),
                    )
                    .map_err(KernelError::HalError)?,
                TerminalType::Display => KernelData::display()
                    .draw_char_at_cursor(data as u8, Some(self.current_color))
                    .map_err(KernelError::DisplayError)?,
            }
        }

        Ok(())
    }

    /// Writes a string to all available terminal interfaces associated with this object.
    ///
    /// This method iterates over all the terminals managed by the current instance
    /// and sends the provided string data to each terminal. The type of terminal
    /// determines how the string is processed and displayed:
    ///
    /// - **`TerminalType::Usart`**: For UART-based terminals, it sends the string
    ///   using a hardware abstraction layer (HAL) interface.
    /// - **`TerminalType::Display`**: For display terminals, it draws the string
    ///   at the current cursor position using the provided color settings.
    ///
    /// # Parameters:
    /// - `data`: A reference to the string slice (`&str`) that needs to be written
    ///   to the terminals.
    ///
    /// # Returns:
    /// - `Ok(())` on success, indicating that the string was written to all terminals
    ///   without issues.
    /// - `Err(KernelError)`: If an error occurs while writing to one of the
    ///   terminals. Errors can be:
    ///     - `KernelError::HalError`: If there are issues with writing to a UART
    ///       terminal via the HAL interface.
    ///     - `KernelError::DisplayError`: If there are issues with drawing the
    ///       string on the display.
    ///
    /// # Error Handling:
    /// Any write failure to a terminal will immediately stop further writes to
    /// subsequent terminals, and this method will propagate the error using the
    /// `Result` type.
    ///
    /// # Notes:
    /// - The `interface_id` vector is used to map each terminal to its corresponding
    ///   HAL communication interface.
    /// - The method applies the `current_color` field when writing strings to
    ///   display terminals.
    /// - This function assumes that `self.terminals` and `self.interface_id` are synchronized,
    ///   meaning their indices align correctly to map the terminals to their respective communication interfaces.
    ///
    /// # Panics:
    /// - This method does not panic, provided that the internal state (e.g.,
    ///   `self.terminals` and `self.interface_id`) is consistent and valid.
    ///
    /// # Preconditions:
    /// - `self.terminals` must contain valid terminal instances.
    /// - `self.interface_id` must have corresponding entries for each terminal.
    ///
    fn write_str(&self, data: &str) -> KernelResult<()> {
        for (i, terminal) in self.terminals.iter().enumerate() {
            match terminal {
                TerminalType::Usart(_) => KernelData::hal()
                    .interface_write(
                        self.interface_id[i],
                        InterfaceWriteActions::UartWrite(UartWriteActions::SendString(data)),
                    )
                    .map_err(KernelError::HalError)?,
                TerminalType::Display => KernelData::display()
                    .draw_string_at_cursor(data, Some(self.current_color))
                    .map_err(KernelError::DisplayError)?,
            }
        }

        Ok(())
    }

    /// Clears the content of all terminals managed by the kernel.
    ///
    /// This function iterates through all terminals registered in the kernel and performs the
    /// appropriate "clear" action based on the terminal type:
    ///
    /// - For `TerminalType::Usart`, it sends the ANSI escape sequence `\x1B[2J\x1B[H`
    ///   to clear the screen and reset the cursor to the home position.
    /// - For `TerminalType::Display`, it utilizes the display's `clear` method to fill
    ///   the screen with a specified background color (in this case, `Colors::Black`).
    ///
    /// ### Errors
    /// If the function fails to send the clear command to any terminal:
    /// - Returns a `KernelError::HalError` if there was an issue with the hardware abstraction layer (HAL) when working with USART terminals.
    /// - Returns a `KernelError::DisplayError` if clearing a display terminal fails.
    ///
    /// ### Returns
    /// - `Ok(())` if all terminals are successfully cleared without error.
    /// - `Err(KernelError)` if there is a failure in clearing one or more terminals.
    ///
    /// ### Notes
    /// This function relies on the `KernelData::hal()` and `KernelData::display()` implementations
    /// to interact with hardware-specific interfaces. Ensure these components are properly initialized
    /// and accessible before invoking this method.
    pub fn clear_terminal(&self) -> KernelResult<()> {
        for (i, terminal) in self.terminals.iter().enumerate() {
            match terminal {
                TerminalType::Usart(_) => KernelData::hal()
                    .interface_write(
                        self.interface_id[i],
                        InterfaceWriteActions::UartWrite(UartWriteActions::SendString(
                            "\x1B[2J\x1B[H",
                        )),
                    )
                    .map_err(KernelError::HalError)?,
                TerminalType::Display => KernelData::display()
                    .clear(Colors::Black)
                    .map_err(KernelError::DisplayError)?,
            }
        }

        Ok(())
    }
}
