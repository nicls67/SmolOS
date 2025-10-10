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
    pub fn new(terminals: Vec<TerminalType, 8>) -> Terminal {
        Terminal {
            interface_id: Vec::from_slice(&[0; MAX_TERMINALS]).unwrap(),
            terminals,
            line_buffer: String::new(),
            state: TerminalState::Stopped,
            escape_seq: EscapeSeqState::NotInEcsSeq,
            cursor_pos: 0,
            current_color: display::Colors::White,
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

    #[inline(always)]
    fn new_line(&self) -> KernelResult<()> {
        self.write_char('\r')?;
        self.write_char('\n')
    }

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
                    .draw_string_at_cursor(
                        str::from_utf8(&[data as u8]).unwrap(),
                        self.current_color,
                    )
                    .map_err(KernelError::DisplayError)?,
            }
        }

        Ok(())
    }

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
                    .draw_string_at_cursor(data, self.current_color)
                    .map_err(KernelError::DisplayError)?,
            }
        }

        Ok(())
    }

    fn clear_terminal(&self) -> KernelResult<()> {
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
