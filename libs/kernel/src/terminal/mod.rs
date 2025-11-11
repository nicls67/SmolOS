mod terminal_wrapper;

use crate::KernelError::TerminalError;
use crate::KernelErrorLevel::Error;
use crate::TerminalType::Usart;
use crate::data::Kernel;
use crate::ident::KERNEL_MASTER_ID;
use crate::terminal::TerminalState::{Display, Prompt, Stopped};
use crate::{KernelResult, SysCallHalActions, SysCallHalArgs, Syscall, syscall};

use crate::terminal::terminal_wrapper::TerminalWrapper;
use display::Colors;
use hal_interface::{BUFFER_SIZE, InterfaceReadAction, InterfaceReadResult};
use heapless::{String, Vec, format};

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

#[derive(Debug, Copy, Clone)]
pub enum TerminalType {
    Usart(&'static str),
    Display,
}

/// Represents the state of a terminal.
///
/// The `TerminalState` enum defines various operational states
/// that a terminal can be in, providing control over its interaction
/// with users and kernel processes.
#[derive(PartialEq, Clone, Copy, Debug)]
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
    terminals: Vec<TerminalWrapper, MAX_TERMINALS>,
}

impl Terminal {
    /// Creates a new `Terminal` instance containing a collection of `TerminalWrapper` objects
    /// initialized with the provided terminal types.
    ///
    /// # Parameters
    /// - `terminals`: A `Vec` containing up to 8 `TerminalType` objects that define the terminals to be wrapped.
    ///
    /// # Returns
    /// - A new instance of the `Terminal` struct.
    ///
    /// # Functionality
    /// - Iterates through each item in the `terminals` vector.
    /// - Wraps each terminal type in a `TerminalWrapper` struct, initializing the following fields:
    ///   - `interface_id`: Set to `0`.
    ///   - `terminal`: The terminal type passed as the input.
    ///   - `line_buffer`: Initialized with an empty `String`.
    ///   - `mode`: Set to `Stopped` (assumes the terminal starts in the stopped state).
    ///   - `cursor_pos`: Set to `0` (initial cursor position).
    ///   - `current_color`: Set to `Colors::White` as the default.
    ///   - `owner`: Set to `None` (no owner initially).
    /// - Pushes the created `TerminalWrapper` into the `terms` vector.
    ///
    /// # Panics
    /// - The function may panic if the `push` operation on the `terms` vector fails.
    ///   This is due to the `.unwrap()` call which asserts that the operation will succeed.
    ///
    pub fn new(terminals: Vec<TerminalType, 8>) -> Terminal {
        let mut terms = Vec::new();
        for terminal in terminals {
            let term_wrapper = TerminalWrapper {
                interface_id: 0,
                terminal,
                line_buffer: String::new(),
                mode: Stopped,
                cursor_pos: 0,
                current_color: Colors::White,
                owner: None,
            };
            terms.push(term_wrapper).unwrap();
        }

        Terminal { terminals: terms }
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
        for terminal in self.terminals.iter_mut() {
            // Only USART terminals are supported in prompt mode
            if let Usart(name) = terminal.terminal {
                // Retrieve interface id if the terminal is stopped
                if terminal.mode == Stopped {
                    syscall(
                        Syscall::Hal(SysCallHalArgs {
                            id: terminal.interface_id,
                            action: SysCallHalActions::GetID(name, &mut terminal.interface_id),
                        }),
                        KERNEL_MASTER_ID,
                    )?;
                }

                // Configure callback for user prompt data
                syscall(
                    Syscall::Hal(SysCallHalArgs {
                        id: terminal.interface_id,
                        action: SysCallHalActions::ConfigureCallback(terminal_prompt_callback),
                    }),
                    KERNEL_MASTER_ID,
                )?;

                // Set mode to prompt
                if terminal.mode != Prompt {
                    terminal.mode = Prompt;
                    terminal.cursor_pos = 0;
                    terminal.new_line()?;
                    terminal.write_char('>')?;
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
        for terminal in self.terminals.iter_mut() {
            // Retrieve interface id if the terminal is stopped
            if terminal.mode == Stopped {
                // We need interface ID only for USART terminals
                if let Usart(name) = terminal.terminal {
                    syscall(
                        Syscall::Hal(SysCallHalArgs {
                            id: terminal.interface_id,
                            action: SysCallHalActions::GetID(name, &mut terminal.interface_id),
                        }),
                        KERNEL_MASTER_ID,
                    )?;
                }
            }

            // Set mode to display
            if terminal.mode != Display {
                terminal.mode = Display;
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
        for terminal in self.terminals.iter() {
            if terminal.mode == Display {
                match format {
                    TerminalFormatting::StrNoFormatting(text) => terminal.write_str(text)?,
                    TerminalFormatting::StrNewLineAfter(text) => {
                        terminal.write_str(text)?;
                        terminal.new_line()?;
                    }
                    TerminalFormatting::StrNewLineBefore(text) => {
                        terminal.new_line()?;
                        terminal.write_str(text)?;
                    }
                    TerminalFormatting::StrNewLineBoth(text) => {
                        terminal.new_line()?;
                        terminal.write_str(text)?;
                        terminal.new_line()?;
                    }
                    TerminalFormatting::Newline => terminal.new_line()?,
                    TerminalFormatting::Char(c) => terminal.write_char(*c)?,
                    TerminalFormatting::Clear => terminal.clear_terminal()?,
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
        for terminal in self.terminals.iter_mut() {
            terminal.current_color = color;
        }
    }

    /// Processes input received from the terminal and handles user commands or interactions.
    ///
    /// # Parameters
    /// - `buffer`: A fixed-size buffer (`Vec<u8, BUFFER_SIZE>`) containing input data, where the first byte represents the character entered by the user.
    /// - `id`: A `usize` representing the identifier of the terminal receiving the input.
    ///
    /// # Returns
    /// - `KernelResult<()>`: Returns `Ok(())` on successful processing or an appropriate error if an issue occurs.
    ///
    /// # Behavior
    /// - The function identifies the terminal associated with the given `id`.
    /// - If the terminal is in prompt mode (`Prompt`):
    ///   - If the first character in the buffer is a return character (`\r`):
    ///     - Attempts to execute the command stored in the terminal's line buffer via `Kernel::apps().start_app`.
    ///         - If the command execution succeeds, it proceeds to the next step.
    ///         - If the command execution fails, writes an error message (`"Error: Command not found"`) to the terminal.
    ///     - Clears the terminal's line buffer, resets the cursor's position, and moves to a new line.
    ///     - Finally, writes a prompt character (`>`) to indicate readiness for a new user command.
    ///   - If the first character is not a return character:
    ///     - Echoes the received character back to the terminal.
    ///     - Appends the echoed character to the terminal's line buffer. If the buffer overflows, it returns an error of type `TerminalError` with a specific error message ("Line buffer overflow").
    ///     - Updates the cursor position by incrementing it by one.
    ///
    /// # Errors
    /// - Returns an error if the line buffer overflows while appending a character.
    /// - Returns any other error encountered while writing to the terminal.
    ///
    pub fn process_input(&mut self, buffer: Vec<u8, BUFFER_SIZE>, id: usize) -> KernelResult<()> {
        // Find the terminal corresponding to the given ID
        let mut terminal = self
            .terminals
            .iter_mut()
            .find(|t| t.interface_id == id)
            .unwrap();

        // If the terminal is in prompt mode
        if terminal.mode == Prompt {
            // If the received character is a return character, process the line
            if buffer[0] == '\r' as u8 {
                // Start the requested command
                match Kernel::apps().start_app(&terminal.line_buffer) {
                    Ok(_) => {}
                    Err(err) => terminal
                        .write_str(format!(256;"\r\n{}",err.to_string()).unwrap().as_str())?,
                };

                // Empty the line buffer and go to a new line
                terminal.line_buffer.clear();
                terminal.cursor_pos = 0;
                terminal.new_line()?;
                terminal.write_char('>')?;
            } else {
                // Echo the received character
                terminal.write_char(buffer[0] as char)?;

                // Store it into the line buffer
                let term_name = terminal.name();
                terminal
                    .line_buffer
                    .push(buffer[0] as char)
                    .map_err(|_| TerminalError(Error, term_name, "Line buffer overflow"))?;
                terminal.cursor_pos += 1;
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
