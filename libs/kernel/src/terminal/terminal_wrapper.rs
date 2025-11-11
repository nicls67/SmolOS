use crate::KernelError::TerminalError;
use crate::KernelErrorLevel::Error;
use crate::TerminalType::Usart;
use crate::ident::KERNEL_MASTER_ID;
use crate::terminal::TerminalState;
use crate::{
    KernelResult, SysCallDisplayArgs, SysCallHalActions, SysCallHalArgs, Syscall, TerminalType,
    syscall,
};
use display::Colors;
use hal_interface::{InterfaceWriteActions, UartWriteActions};
use heapless::String;

#[derive(Debug)]
pub struct TerminalWrapper {
    pub interface_id: usize,
    pub terminal: TerminalType,
    pub line_buffer: String<256>,
    pub mode: TerminalState,
    pub cursor_pos: usize,
    pub current_color: Colors,
    pub owner: Option<u32>,
}

impl TerminalWrapper {
    /// Checks if the given caller ID has the correct rights to access or perform operations
    /// on the kernel object. This method verifies ownership or privileged access.
    ///
    /// ### Parameters:
    /// - `caller_id` (`u32`): The ID of the entity attempting to access the resource.
    ///
    /// ### Returns:
    /// - `KernelResult<()>`:
    ///   - `Ok(())` if the `caller_id` has the required rights.
    ///   - `Err(TerminalError)`: If the `caller_id` does not have the necessary rights.
    ///
    /// ### Ownership Rules:
    /// - Access is granted if:
    ///   1. The caller ID matches the owner of the resource.
    ///   2. The caller ID matches the predefined `KERNEL_MASTER_ID` which represents
    ///      a superuser or privileged access level.
    /// - If the resource does not have an owner (`self.owner` is `None`), access is universally granted.
    ///
    /// ### Errors:
    /// - Returns a `TerminalError` with a message of "Permission denied" if:
    ///   - The `caller_id` doesn't match the owner ID, and
    ///   - The `caller_id` is not equal to `KERNEL_MASTER_ID`.
    ///
    fn check_rights(&self, caller_id: u32) -> KernelResult<()> {
        match self.owner {
            Some(owner) => {
                if owner == caller_id || caller_id == KERNEL_MASTER_ID {
                    Ok(())
                } else {
                    Err(TerminalError(Error, self.name(), "Permission denied"))
                }
            }
            None => Ok(()),
        }
    }

    /// Writes a new line to the output by sequentially writing a carriage return (`'\r'`)
    /// and a newline (`'\n'`).
    ///
    /// # Returns
    ///
    /// * `KernelResult<()>` - Returns `Ok(())` if both characters are successfully written,
    /// or an error wrapped in `KernelResult` if the write operation fails.
    ///
    /// # Behavior
    /// - The method first writes a `'\r'` character followed by a `'\n'`.
    /// - The `#[inline(always)]` attribute suggests to the compiler that this method
    ///   should always be inlined to improve performance.
    ///
    /// # Errors
    /// Returns an error if any of the two character write operations fail.
    ///
    #[inline(always)]
    pub(crate) fn new_line(&self) -> KernelResult<()> {
        self.write_char('\r')?;
        self.write_char('\n')
    }

    /// Writes a single character to the terminal interface.
    ///
    /// This function sends a character to the terminal associated with the interface.
    /// Depending on the terminal type (USART or Display), it invokes the appropriate
    /// system call to perform the write operation. The character's appearance and behavior
    /// may vary depending on the terminal's underlying driver and implementation.
    ///
    /// # Arguments
    ///
    /// * `data` - The character to be written to the terminal.
    ///
    /// # Returns
    ///
    /// * `KernelResult<()>` - Returns an empty result on success or an error if the write operation fails.
    ///
    /// # Behavior
    ///
    /// - If the terminal type is `Usart(_)`, the character is sent over UART via a syscall using
    ///   the `SysCallHalActions::Write` action with the `UartWriteActions::SendChar` operation.
    /// - If the terminal type is `TerminalType::Display`, the character is displayed using
    ///   `SysCallDisplayArgs::WriteCharAtCursor`, potentially with the current terminal color.
    ///
    /// # Errors
    ///
    /// If the syscall fails (e.g., due to an invalid interface ID, unsupported operation, or other
    /// underlying issues), the function will return an appropriate error wrapped in a `KernelResult`.
    ///
    pub(crate) fn write_char(&self, data: char) -> KernelResult<()> {
        match self.terminal {
            Usart(_) => syscall(
                Syscall::Hal(SysCallHalArgs {
                    id: self.interface_id,
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

    /// Writes a string to the specified output interface.
    ///
    /// This function handles writing a string to one of two terminal types: `Usart` or `Display`.
    /// Depending on the terminal type, it invokes the appropriate system call to perform the write
    /// operation. If the terminal is of type `Usart`, the string is sent via UART, and if the terminal
    /// is of type `Display`, the string is written to the display at the current cursor position,
    /// possibly with an associated text color.
    ///
    /// # Parameters
    /// - `data`: A reference to the string slice (`&str`) to be written to the terminal.
    ///
    /// # Returns
    /// - `KernelResult<()>`: Returns `Ok(())` if the string was successfully written to the
    ///   specified interface, or an error if the system call fails.
    ///
    /// # System Calls
    /// - For `Usart`, the function makes a `Syscall::Hal` system call with a write action (`UartWrite`).
    /// - For `Display`, the function makes a `Syscall::Display` system call to write the string
    ///   to the display at the current cursor position.
    ///
    /// # Errors
    /// - This function propagates any errors generated by the `syscall` function, which may occur
    ///   if the system call fails to execute properly.
    ///
    pub(crate) fn write_str(&self, data: &str) -> KernelResult<()> {
        match self.terminal {
            Usart(_) => syscall(
                Syscall::Hal(SysCallHalArgs {
                    id: self.interface_id,
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

    /// Clears the terminal screen based on the terminal type.
    ///
    /// This function handles clearing the screen for different terminal types:
    /// - For `Usart`: Sends an ANSI escape sequence (`\x1B[2J\x1B[H`) to clear the screen.
    /// - For `TerminalType::Display`: Invokes a system call to clear the display with a specified background color (`Colors::Black`).
    ///
    /// # Returns
    ///
    /// Returns a `KernelResult<()>`, indicating the success (`Ok`) or failure (`Err`) of the operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying system call (`syscall`) fails for either terminal type.
    ///
    pub fn clear_terminal(&self) -> KernelResult<()> {
        match self.terminal {
            Usart(_) => syscall(
                Syscall::Hal(SysCallHalArgs {
                    id: self.interface_id,
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

    /// This method retrieves the name of the terminal associated with the instance.
    ///
    /// # Returns
    ///
    /// * A `&'static str` representing the name of the terminal.
    ///   - If the terminal is of type `Usart`, it returns the associated name of the `Usart` terminal.
    ///   - If the terminal is of type `Display`, it returns the static string `"Display"`.
    ///
    /// # Note
    ///
    /// Ensure the terminal type is properly handled within the `match` statement.
    pub fn name(&self) -> &'static str {
        match self.terminal {
            Usart(n) => n,
            TerminalType::Display => "Display",
        }
    }
}
