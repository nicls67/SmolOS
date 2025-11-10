use crate::KernelError::TerminalError;
use crate::KernelErrorLevel::Error;
use crate::TerminalType::Usart;
use crate::data::Kernel;
use crate::ident::KERNEL_MASTER_ID;
use crate::terminal::TerminalState::{Display, Prompt, Stopped};
use crate::{
    KernelResult, SysCallDisplayArgs, SysCallHalActions, SysCallHalArgs, Syscall, syscall,
};
use display::Colors;
use hal_interface::{
    BUFFER_SIZE, InterfaceReadAction, InterfaceReadResult, InterfaceWriteActions, UartWriteActions,
};
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
#[derive(PartialEq, Clone, Copy)]
enum TerminalState {
    /// Terminal is stopped
    Stopped,
    /// Terminal is in prompt mode
    Prompt,
    /// Terminal is in display-only mode
    Display,
}

const MAX_TERMINALS: usize = 8;

pub struct Terminal {
    interface_id: Vec<usize, MAX_TERMINALS>,
    terminals: Vec<TerminalType, MAX_TERMINALS>,
    line_buffer: Vec<String<256>, MAX_TERMINALS>,
    mode: Vec<TerminalState, MAX_TERMINALS>,
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
    /// - `cursor_pos`: Initialized to `0`.
    /// - `current_color`: Set to `Colors::White`.
    ///
    /// # Panics
    /// This function will panic if the creation of the ` interface_id ` vector fails, which is unlikely
    /// unless system memory constraints are exceeded.
    ///
    pub fn new(terminals: Vec<TerminalType, 8>) -> Terminal {
        Terminal {
            interface_id: Vec::from_slice(&[0; MAX_TERMINALS]).unwrap(),
            terminals,
            line_buffer: Vec::from_slice(&[const { String::new() }; MAX_TERMINALS]).unwrap(),
            mode: Vec::from_slice(&[Stopped; MAX_TERMINALS]).unwrap(),
            cursor_pos: 0,
            current_color: Colors::White,
        }
    }

    /// Retrieves the name of a terminal based on its index.
    ///
    /// # Arguments
    ///
    /// * `term_idx` - The index of the terminal in the `terminals` vector.
    ///
    /// # Returns
    ///
    /// A string slice (`&str`) representing the name of the terminal.
    /// If the terminal is of type `Usart`, the corresponding name is returned.
    /// If the terminal is of type `Display`, the string "Display" is returned.
    ///
    /// # Panics
    ///
    /// This function may panic if `term_idx` is out of bounds for the `terminals` vector.
    ///
    /// # Note
    ///
    /// Ensure that `term_idx` is within the valid range of terminal indices to avoid
    /// runtime panics.
    fn name(&self, term_idx: usize) -> &'static str {
        match self.terminals[term_idx] {
            Usart(n) => n,
            TerminalType::Display => "Display",
        }
    }

    /// Configures terminal interfaces to operate in prompt mode, enabling user input capabilities.
    ///
    /// # Description
    /// This function iterates over all terminal interfaces managed by the object.
    /// It ensures that only USART terminals are set to prompt mode, as these are suited for user interaction.
    /// If a terminal is in the `Stopped` state, the function attempts to retrieve or update the interface ID
    /// by performing a system call. Once the required interface IDs are resolved, the function transitions
    /// those terminals not already in `Prompt` mode into that mode. Prompt mode setups involve resetting the escape
    /// sequence state, positioning the cursor, starting a new line, and displaying a `>` prompt character.
    ///
    /// # Behavior
    /// - Only terminals of type `Usart` are processed.
    /// - Any terminal in the `Stopped` state will trigger a system call to retrieve the interface ID.
    /// - Terminals already in `Prompt` mode are left unchanged.
    /// - For terminals transitioning to `Prompt` mode:
    ///   * Resets escape sequence state.
    ///   * Resets the cursor position to the start.
    ///   * Outputs a new line and writes the `>` character as a prompt
    pub fn set_prompt_mode(&mut self) -> KernelResult<()> {
        for (i, terminal) in self.terminals.iter().enumerate() {
            // Only USART terminals are supported in prompt mode
            if let Usart(name) = terminal {
                // Retrieve interface id if the terminal is stopped
                if self.mode[i] == Stopped {
                    syscall(
                        Syscall::Hal(SysCallHalArgs {
                            id: self.interface_id[i],
                            action: SysCallHalActions::GetID(name, &mut self.interface_id[i]),
                        }),
                        KERNEL_MASTER_ID,
                    )?;
                }

                // Configure callback for user prompt data
                syscall(
                    Syscall::Hal(SysCallHalArgs {
                        id: self.interface_id[i],
                        action: SysCallHalActions::ConfigureCallback(terminal_prompt_callback),
                    }),
                    KERNEL_MASTER_ID,
                )?;

                // Set mode to prompt
                if self.mode[i] != Prompt {
                    self.mode[i] = Prompt;
                    self.cursor_pos = 0;
                    self.new_line(i)?;
                    self.write_char('>', i)?;
                }
            }
        }

        Ok(())
    }

    /// Sets the display mode for all terminals managed by the system.
    ///
    /// This method iterates through each terminal, identifies terminals that are currently
    /// in the `Stopped` state, and performs the necessary actions to transition them to the
    /// `Display` mode. If a terminal is of type `Usart`, it retrieves its interface ID
    /// by making a system call using the `Syscall::Hal` API.
    ///
    /// # Behavior
    /// - For each terminal that is in the `Stopped` state:
    ///   1. If the terminal is of type `Usart`, the interface ID is queried and updated.
    ///   2. The terminal's mode is set to `Display`.
    /// - Terminals that are not in the `Stopped` state are ignored.
    ///
    /// # Errors
    /// This function can return a `KernelResult::Err` if the system call invoked to
    /// retrieve the interface ID fails. In such a case, the error will propagate to the caller.
    ///
    /// # Parameters
    /// - `&mut self`: A mutable reference to the instance of the containing structure. This
    ///   is required to modify the terminal modes and interface IDs.
    ///
    /// # Returns
    /// - `KernelResult<()>`: Indicates whether the display mode update operation was successful.
    ///   Returns `Ok(())` if successful, or the appropriate error if an issue occurs.
    ///
    /// # System Calls
    /// - Invokes `syscall` with `Syscall::Hal` whenever an `Usart` terminal in the `Stopped`
    ///   state requires its interface ID to be retrieved.
    ///
    pub fn set_display_mode(&mut self) -> KernelResult<()> {
        for (i, terminal) in self.terminals.iter().enumerate() {
            // Retrieve interface id if the terminal is stopped
            if self.mode[i] == Stopped {
                // We need interface ID only for USART terminals
                if let TerminalType::Usart(name) = terminal {
                    syscall(
                        Syscall::Hal(SysCallHalArgs {
                            id: self.interface_id[i],
                            action: SysCallHalActions::GetID(name, &mut self.interface_id[i]),
                        }),
                        KERNEL_MASTER_ID,
                    )?;
                }
            }

            // Set mode to display
            if self.mode[i] != Display {
                self.mode[i] = Display;
            }
        }
        Ok(())
    }

    ///
    /// Writes data to terminals based on the specified formatting.
    ///
    /// This function iterates through all terminals managed by the current instance and writes
    /// formatted content to terminals that are in the `Display` mode. The formatting can include raw text,
    /// text with newlines before or after, single characters, clearing the terminal, and others.
    ///
    /// # Arguments
    ///
    /// * `format` - A reference to a `TerminalFormatting` enum that specifies the type of content or
    ///   action to be processed and displayed on the terminal(s).
    ///
    /// # Terminal Formatting Variants
    ///
    /// - `TerminalFormatting::StrNoFormatting(text)`:
    ///     Writes the provided string (`text`) to the terminal directly without additional newlines.
    /// - `TerminalFormatting::StrNewLineAfter(text)`:
    ///     Writes the string `text` followed by a new line.
    /// - `TerminalFormatting::StrNewLineBefore(text)`:
    ///     Writes a new line first, then writes the string `text`.
    /// - `TerminalFormatting::StrNewLineBoth(text)`:
    ///     Writes a new line, then the string `text`, and then another new line.
    /// - `TerminalFormatting::Newline`:
    ///     Writes a new line to the terminal.
    /// - `TerminalFormatting::Char(c)`:
    ///     Writes a single character (`c`) to the terminal.
    /// - `TerminalFormatting::Clear`:
    ///     Clears the contents of the terminal.
    ///
    /// # Modes
    ///
    /// The function only processes terminals that are in `Display` mode. Terminals in other modes are skipped.
    ///
    /// # Errors
    ///
    /// Returns a `KernelResult` indicating success (`Ok(())`) or failure. A failure might occur due to
    /// I/O errors, system constraints, or other kernel-related issues during terminal operations.
    ///
    /// The above example writes "Hello, world!" to all terminals in `Display` mode, with a newline
    /// before and after the string.
    ///
    /// # Notes
    ///
    /// - This function is designed for multi-terminal systems, allowing content to be written to
    ///   multiple terminals based on their index and mode.
    /// - Ensure proper error handling for the returned `KernelResult`, as terminal operations may fail.
    pub fn write(&self, format: &TerminalFormatting) -> KernelResult<()> {
        for term_idx in 0..self.terminals.len() {
            if self.mode[term_idx] == Display {
                match format {
                    TerminalFormatting::StrNoFormatting(text) => self.write_str(text, term_idx)?,
                    TerminalFormatting::StrNewLineAfter(text) => {
                        self.write_str(text, term_idx)?;
                        self.new_line(term_idx)?;
                    }
                    TerminalFormatting::StrNewLineBefore(text) => {
                        self.new_line(term_idx)?;
                        self.write_str(text, term_idx)?;
                    }
                    TerminalFormatting::StrNewLineBoth(text) => {
                        self.new_line(term_idx)?;
                        self.write_str(text, term_idx)?;
                        self.new_line(term_idx)?;
                    }
                    TerminalFormatting::Newline => self.new_line(term_idx)?,
                    TerminalFormatting::Char(c) => self.write_char(*c, term_idx)?,
                    TerminalFormatting::Clear => self.clear_terminal(term_idx)?,
                }
            }
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
    pub fn set_color(&mut self, color: Colors) {
        self.current_color = color;
    }

    /// Inserts a new line in the specified terminal by writing a carriage return (`\r`)
    /// followed by a line feed (`\n`) character.
    ///
    /// # Parameters
    /// - `term_idx`: The index
    #[inline(always)]
    fn new_line(&self, term_idx: usize) -> KernelResult<()> {
        self.write_char('\r', term_idx)?;
        self.write_char('\n', term_idx)
    }

    /// Writes a character to a terminal specified by the `term_idx`.
    ///
    /// This function sends the given character (`data`) to the appropriate terminal device,
    /// identified by its index (`term_idx`). The terminals could represent different types
    /// of output devices, such as a USART or display. The behavior of the function depends on
    /// the type of terminal being interacted with.
    ///
    /// # Parameters
    /// - `data`: The character to write to the terminal.
    /// - `term_idx`: The index of the terminal in the `terminals` array to which the character is written.
    ///
    /// # Returns
    /// - `KernelResult<()>`: Returns `Ok(())` if the character is successfully written. Returns an error
    ///   if the operation fails.
    ///
    /// # Behavior
    /// - If the terminal type is `Usart`, the function utilizes a HAL system call to send the character
    ///   as a UART signal.
    /// - If the terminal type is `Display`, the function uses a display system call to write the character
    ///   at the cursor position with an optional color (`current_color`).
    ///
    /// # Errors
    /// Returns an error wrapped in `KernelResult` if the system call fails or if an invalid terminal index
    /// is accessed.
    ///
    /// # Notes
    /// - Ensure that `term_idx` corresponds to a valid terminal in the `terminals` array.
    /// - The implementation assumes that `self.terminals` and `self.interface_id` are properly
    ///   initialized before calling this function.
    fn write_char(&self, data: char, term_idx: usize) -> KernelResult<()> {
        match self.terminals[term_idx] {
            Usart(_) => syscall(
                Syscall::Hal(SysCallHalArgs {
                    id: self.interface_id[term_idx],
                    action: SysCallHalActions::Write(InterfaceWriteActions::UartWrite(
                        UartWriteActions::SendChar(data as u8),
                    )),
                }),
                KERNEL_MASTER_ID,
            )?,
            TerminalType::Display => syscall(
                Syscall::Display(SysCallDisplayArgs::WriteCharAtCursor(
                    data,
                    Some(self.current_color),
                )),
                KERNEL_MASTER_ID,
            )?,
        }

        Ok(())
    }

    /// Writes a string to the specified terminal interface.
    ///
    /// Depending on the terminal type, this function performs the appropriate
    /// system call to write the string. It supports writing to both USART (UART)
    /// and Display terminal interfaces.
    ///
    /// # Parameters
    /// - `&self`: The current context or instance containing terminal configurations.
    /// - `data: &str`: The string slice to be written to the terminal.
    /// - `term_idx: usize`: The zero-based index of the terminal to which the string
    ///   will be written. `term_idx` should correspond to an entry in `self.terminals`.
    ///
    /// # Returns
    /// - `KernelResult<()>`: Returns `Ok(())` if the operation completes successfully,
    ///   otherwise returns an error encapsulated within `KernelResult`.
    ///
    /// # Behavior
    /// - If the terminal at the given index is of type `Usart`, the function invokes
    ///   a system call for UART write action using the provided string.
    /// - If the terminal is of type `Display`, the function makes a system call to
    ///   write the string at the current cursor position with the specified color.
    ///
    /// # Errors
    /// - Returns an error if the system call fails during the write operation, which
    ///   will be propagated as part of the `KernelResult`.
    ///
    fn write_str(&self, data: &str, term_idx: usize) -> KernelResult<()> {
        match self.terminals[term_idx] {
            Usart(_) => syscall(
                Syscall::Hal(SysCallHalArgs {
                    id: self.interface_id[term_idx],
                    action: SysCallHalActions::Write(InterfaceWriteActions::UartWrite(
                        UartWriteActions::SendString(data),
                    )),
                }),
                KERNEL_MASTER_ID,
            )?,
            TerminalType::Display => syscall(
                Syscall::Display(SysCallDisplayArgs::WriteStrAtCursor(
                    data,
                    Some(self.current_color),
                )),
                KERNEL_MASTER_ID,
            )?,
        }

        Ok(())
    }

    /// Clears the terminal display for a given terminal index.
    ///
    /// This function sends a clear screen command to the specified terminal,
    /// either a USART-based terminal or a display-based terminal. It resets
    /// the terminal to a clean state.
    ///
    /// # Arguments
    ///
    /// * `term_idx` - The index of the terminal to be cleared. This index should
    ///   correspond to an entry in the `self.terminals` array.
    ///
    /// # Behavior
    ///
    /// - For `Usart` terminals, it sends the ANSI escape sequence `\x1B[2J\x1B[H`,
    ///   which clears the screen and moves the cursor to the home position.
    /// - For `Display` terminals, it issues a display clear command and sets the
    ///   background to black.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - On successful execution of the terminal clear command.
    /// * `Err(KernelError)` - If an error occurs during the system call, it will
    ///   bubble up to the caller.
    ///
    /// # Errors
    ///
    /// If the syscall fails for any reason, this function will return a
    /// `KernelResult` error variant indicating the failure.
    ///
    pub fn clear_terminal(&self, term_idx: usize) -> KernelResult<()> {
        match self.terminals[term_idx] {
            Usart(_) => syscall(
                Syscall::Hal(SysCallHalArgs {
                    id: self.interface_id[term_idx],
                    action: SysCallHalActions::Write(InterfaceWriteActions::UartWrite(
                        UartWriteActions::SendString("\x1B[2J\x1B[H"),
                    )),
                }),
                KERNEL_MASTER_ID,
            )?,
            TerminalType::Display => syscall(
                Syscall::Display(SysCallDisplayArgs::Clear(Colors::Black)),
                KERNEL_MASTER_ID,
            )?,
        }

        Ok(())
    }

    /// Processes input received for a terminal and handles it based on the mode and input buffer.
    ///
    /// # Parameters
    /// - `buffer`: A `Vec<u8, BUFFER_SIZE>` containing the input data to be processed. This typically
    ///   represents a single character or multiple characters received.
    /// - `id`: A `usize` that represents the unique identifier of the terminal to which the input belongs.
    ///
    /// # Returns
    /// - A `KernelResult<()>`, indicating success or failure of the operation. In case of errors, a
    ///   relevant terminal error is returned.
    ///
    /// # Functionality
    /// - Locates the terminal corresponding to the given `id` from the list of `interface_id`s.
    /// - If the terminal is in the `Prompt` mode:
    ///   - Checks the contents of the `buffer`.
    ///   - If the input is a carriage return (`\r`), it clears the terminal's line buffer, resets the
    ///     cursor position, moves to a new line, and writes a new prompt (`>`).
    ///   - If the input is any other character, echoes the character back to the terminal, appends it
    ///     to the terminal's line buffer, and updates the cursor position.
    ///   - Handles potential errors during line buffer updates, such as buffer overflow.
    ///
    /// # Errors
    /// - If the `line_buffer` overflows (exceeds its capacity), a `TerminalError` is returned with the
    ///   following context:
    ///   - Error Type: `Error`
    ///   - Terminal name or identifier
    ///   - Description: `"Line buffer overflow"`
    ///
    pub fn process_input(&mut self, buffer: Vec<u8, BUFFER_SIZE>, id: usize) -> KernelResult<()> {
        // Find the terminal corresponding to the given ID
        let terminal_idx = self.interface_id.iter().position(|&x| x == id).unwrap();

        // If the terminal is in prompt mode
        if self.mode[terminal_idx] == Prompt {
            // If the received character is a return character, process the line
            if buffer[0] == '\r' as u8 {
                // Currently we only empty the line buffer and go to a new line
                self.line_buffer[terminal_idx].clear();
                self.cursor_pos = 0;
                self.new_line(terminal_idx)?;
                self.write_char('>', terminal_idx)?;
            } else {
                // Echo the received character
                self.write_char(buffer[0] as char, terminal_idx)?;

                // Store it into the line buffer
                let term_name = self.name(terminal_idx);
                self.line_buffer[terminal_idx]
                    .push(buffer[0] as char)
                    .map_err(|_| TerminalError(Error, term_name, "Line buffer overflow"))?;
                self.cursor_pos += 1;
            }
        }

        Ok(())
    }
}

/// A callback function for terminal prompts that reads input data from a specified interface and processes it.
///
/// # Parameters
/// - `id: u8`: The identifier for the specific hardware interface to read input from.
///
/// # Description
/// This function performs the following operations:
/// 1. Initializes an ` InterfaceReadResult ` to store the result of a read operation.
/// 2. Executes a syscall to the kernel to read data from the hardware interface identified by `id`.
/// 3. If the syscall is successful and the read result contains a buffer, the data in the buffer
///    is passed to the terminal module for processing.
/// 4. Handles any errors that occur during the syscall or terminal input processing by invoking
///    the kernel's error handler.
///
/// # Error Handling
/// - If the syscall to read input data fails, the kernel's error handler is invoked with the error.
/// - If the terminal input processing fails, the kernel's error handler is invoked with the error.
///
/// # Notes
/// - The function uses the `Kernel::terminal()` API to access the terminal for input processing.
/// - The `Kernel::errors()` API is used to handle any errors that occur.
///
/// # Safety
/// This function interacts with external systems (e.g., hardware and kernel components)
/// and relies on the validity and stability of the provided `id` and kernel subsystems.
pub extern "C" fn terminal_prompt_callback(id: u8) {
    let mut result = InterfaceReadResult::BufferRead(Vec::new());
    match syscall(
        Syscall::Hal(SysCallHalArgs {
            id: id as usize,
            action: SysCallHalActions::Read(InterfaceReadAction::BufferRead, &mut result),
        }),
        KERNEL_MASTER_ID,
    ) {
        Ok(()) => {
            if let InterfaceReadResult::BufferRead(buffer) = result {
                match Kernel::terminal().process_input(buffer, id as usize) {
                    Ok(_) => {}
                    Err(e) => Kernel::errors().error_handler(&e),
                }
            }
        }
        Err(e) => Kernel::errors().error_handler(&e),
    }
}
