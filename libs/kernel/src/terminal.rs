use crate::data::Kernel;
use crate::{KernelError, KernelResult};
use hal_interface::{InterfaceActions, UartWriteActions};
use heapless::String;

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

pub enum TerminalType {
    Usart,
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

pub struct Terminal {
    pub name: &'static str,
    interface_id: Option<usize>,
    terminal: TerminalType,
    line_buffer: String<256>,
    state: TerminalState,
    escape_seq: EscapeSeqState,
    cursor_pos: usize,
}

impl Terminal {
    pub fn new(name: &'static str, term: TerminalType) -> Terminal {
        Terminal {
            name,
            interface_id: None,
            terminal: term,
            line_buffer: String::new(),
            state: TerminalState::Stopped,
            escape_seq: EscapeSeqState::NotInEcsSeq,
            cursor_pos: 0,
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
        self.interface_id = Some(
            Kernel::hal()
                .get_interface_id(self.name)
                .map_err(KernelError::HalError)?,
        );
        if self.state != TerminalState::Kernel {
            self.state = TerminalState::Kernel
        }
        Ok(())
    }

    #[inline(always)]
    fn new_line(&self) -> KernelResult<()> {
        self.write_char('\r')?;
        self.write_char('\n')
    }

    pub fn write(&self, format: &TerminalFormatting) -> KernelResult<()> {
        if self.state == TerminalState::Kernel {
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
        match self.terminal {
            TerminalType::Usart => Kernel::hal()
                .interface_action(
                    self.interface_id.unwrap(),
                    InterfaceActions::UartWrite(UartWriteActions::SendChar(data as u8)),
                )
                .map_err(KernelError::HalError),
        }
    }

    fn write_str(&self, data: &str) -> KernelResult<()> {
        match self.terminal {
            TerminalType::Usart => Kernel::hal()
                .interface_action(
                    self.interface_id.unwrap(),
                    InterfaceActions::UartWrite(UartWriteActions::SendString(data)),
                )
                .map_err(KernelError::HalError),
        }
    }

    #[inline(always)]
    fn clear_terminal(&self) -> KernelResult<()> {
        self.write_str("\x1B[2J\x1B[H")
    }
}
