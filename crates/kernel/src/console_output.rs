use crate::console_output::ConsoleOutputType::{Display, Usart};
use crate::data::Kernel;
use crate::ident::KERNEL_MASTER_ID;
use crate::{KernelError, syscall_devices};

use crate::{KernelResult, SysCallDisplayArgs, SysCallHalActions, syscall_display, syscall_hal};
use display::Colors;
use hal_interface::{InterfaceWriteActions, UartWriteActions};

/// Console output formatting directives used by higher-level console printing APIs.
///
/// This enum describes how a given string or character should be emitted to the current
/// console output (USART or Display), including whether to surround it with newlines
/// or clear the terminal.
///
/// Note: This enum only models formatting intent; applying these directives is handled
/// elsewhere.
pub enum ConsoleFormatting<'a> {
    /// No formatting is done.
    StrNoFormatting(&'a str),
    /// New line is added after write.
    StrNewLineAfter(&'a str),
    /// New line is added before write.
    StrNewLineBefore(&'a str),
    /// New lines are added before and after write.
    StrNewLineBoth(&'a str),
    /// Only adds a new line.
    Newline,
    /// Writes a single character.
    Char(char),
    /// Clears the terminal.
    Clear,
}

/// The destination type for console output.
///
/// - `Usart(&'static str)` targets a named HAL UART/USART interface.
/// - `Display` targets the system display device.
#[derive(Debug)]
pub enum ConsoleOutputType {
    /// Output through a UART/USART HAL interface, identified by name.
    Usart(&'static str),
    /// Output through the display device.
    Display,
}

#[derive(Debug)]
/// A locked console output target (USART or Display) with associated formatting state.
///
/// `ConsoleOutput` represents an exclusive handle to a concrete output destination.
/// It is created via [`ConsoleOutput::new`] which locks the underlying resource
/// (a named HAL UART/USART interface or the display device) using `KERNEL_MASTER_ID`.
///
/// The struct also tracks the `current_color` used for display rendering (ignored for USART).
///
/// Call [`ConsoleOutput::release`] to unlock the underlying destination when done.
pub struct ConsoleOutput {
    pub interface_id: Option<usize>,
    pub output: ConsoleOutputType,
    pub current_color: Colors,
}

impl ConsoleOutput {
    /// Creates a new [`ConsoleOutput`] targeting the given output destination.
    ///
    /// This constructor initializes the struct with no locked interface/device
    /// (`interface_id` is set to `None`). Call [`ConsoleOutput::initialize`] to
    /// acquire the underlying lock before writing.
    ///
    /// # Parameters
    /// - `output`: The destination to write to (USART interface or Display).
    /// - `current_color`: The active display color used when `output` is `Display`
    ///   (ignored for USART).
    ///
    /// # Returns
    /// - `ConsoleOutput`.
    pub fn new(output: ConsoleOutputType, current_color: Colors) -> Self {
        ConsoleOutput {
            interface_id: None,
            output,
            current_color,
        }
    }

    /// Initializes (locks) the configured console output destination.
    ///
    /// For [`ConsoleOutputType::Usart`], this resolves the HAL interface ID from the interface
    /// name, stores it in [`ConsoleOutput::interface_id`], and acquires an exclusive lock on
    /// that interface using [`KERNEL_MASTER_ID`].
    ///
    /// For [`ConsoleOutputType::Display`], this acquires an exclusive lock on the display
    /// device using [`KERNEL_MASTER_ID`].
    ///
    /// # Returns
    /// - `Ok(())` if the destination is successfully resolved (USART only) and locked.
    ///
    /// # Errors
    /// - Returns [`KernelError::HalError`] if resolving or locking the USART interface fails.
    /// - Propagates any error returned by [`Kernel::devices().lock`] when locking the display.
    pub fn initialize(&mut self) -> KernelResult<()> {
        if let ConsoleOutputType::Usart(name) = self.output {
            // Get id for interface
            self.interface_id = Some(
                Kernel::hal()
                    .get_interface_id(name)
                    .map_err(KernelError::HalError)?,
            );

            // Try to lock the interface
            Kernel::hal()
                .lock_interface(self.interface_id.unwrap(), KERNEL_MASTER_ID)
                .map_err(KernelError::HalError)?;
        } else {
            // Try to lock the display device
            Kernel::devices().lock(crate::DeviceType::Display, KERNEL_MASTER_ID)?;
        }

        Ok(())
    }

    /// Writes a CRLF newline sequence (`'\r'` then `'\n'`) to the configured output.
    ///
    /// # Returns
    /// - `Ok(())` if both characters are written successfully.
    ///
    /// # Errors
    /// Propagates any error returned by [`ConsoleOutput::write_char`] for either character.
    #[inline(always)]
    pub(crate) fn new_line(&self) -> KernelResult<()> {
        self.write_char('\r')?;
        self.write_char('\n')
    }

    /// Writes a single character to the configured output.
    ///
    /// For USART output, the character is sent as a single byte (`u8`) to the HAL UART driver.
    /// For Display output, the character is written at the current cursor position using
    /// `current_color`.
    ///
    /// # Parameters
    /// - `data`: The character to write.
    ///
    /// # Returns
    /// - `Ok(())` if the write syscall succeeds.
    ///
    /// # Errors
    /// Returns an error if the underlying syscall fails:
    /// - For USART: errors from `syscall_hal(...)` are propagated.
    /// - For Display: errors from `syscall_display(...)` are propagated.
    pub(crate) fn write_char(&self, data: char) -> KernelResult<()> {
        match self.output {
            Usart(_) => syscall_hal(
                self.interface_id.unwrap(),
                SysCallHalActions::Write(InterfaceWriteActions::UartWrite(
                    UartWriteActions::SendChar(data as u8),
                )),
                KERNEL_MASTER_ID,
            )?,
            Display => syscall_display(
                SysCallDisplayArgs::WriteCharAtCursor(data, Some(self.current_color)),
                KERNEL_MASTER_ID,
            )?,
        }

        Ok(())
    }

    /// Writes a string slice to the configured output.
    ///
    /// For USART output, the string is passed to the HAL UART driver for transmission.
    /// For Display output, the string is written at the current cursor position using
    /// `current_color`.
    ///
    /// # Parameters
    /// - `data`: The string slice to write.
    ///
    /// # Returns
    /// - `Ok(())` if the write syscall succeeds.
    ///
    /// # Errors
    /// Returns an error if the underlying syscall fails:
    /// - For USART: errors from `syscall_hal(...)` are propagated.
    /// - For Display: errors from `syscall_display(...)` are propagated.
    pub(crate) fn write_str(&self, data: &str) -> KernelResult<()> {
        match self.output {
            Usart(_) => syscall_hal(
                self.interface_id.unwrap(),
                SysCallHalActions::Write(InterfaceWriteActions::UartWrite(
                    UartWriteActions::SendString(data),
                )),
                KERNEL_MASTER_ID,
            )?,
            Display => syscall_display(
                SysCallDisplayArgs::WriteStrAtCursor(data, Some(self.current_color)),
                KERNEL_MASTER_ID,
            )?,
        }

        Ok(())
    }

    /// Clears the terminal or display.
    ///
    /// - For USART output, emits the ANSI escape sequence `ESC[2JESC[H` to clear the screen
    ///   and move the cursor to the home position.
    /// - For Display output, clears the display using a black background.
    ///
    /// # Returns
    /// - `Ok(())` if the clear operation succeeds.
    ///
    /// # Errors
    /// Returns an error if the underlying syscall fails:
    /// - For USART: errors from `syscall_hal(...)` are propagated.
    /// - For Display: errors from `syscall_display(...)` are propagated.
    pub fn clear_terminal(&self) -> KernelResult<()> {
        match self.output {
            Usart(_) => syscall_hal(
                self.interface_id.unwrap(),
                SysCallHalActions::Write(InterfaceWriteActions::UartWrite(
                    UartWriteActions::SendString("\x1B[2J\x1B[H"),
                )),
                KERNEL_MASTER_ID,
            )?,
            Display => syscall_display(SysCallDisplayArgs::Clear(Colors::Black), KERNEL_MASTER_ID)?,
        }

        Ok(())
    }

    /// Returns a human-readable name for the configured output destination.
    ///
    /// # Returns
    /// - For [`ConsoleOutputType::Usart`], returns the interface name.
    /// - For [`ConsoleOutputType::Display`], returns `"Display"`.
    pub fn name(&self) -> &'static str {
        match self.output {
            Usart(n) => n,
            Display => "Display",
        }
    }

    /// Releases/unlocks the currently held console output destination.
    ///
    /// This undoes the exclusive lock acquired by [`ConsoleOutput::new`]:
    /// - For [`ConsoleOutputType::Usart`], unlocks the underlying peripheral interface
    ///   associated with `interface_id`.
    /// - For [`ConsoleOutputType::Display`], unlocks the display device.
    ///
    /// After calling this, the `ConsoleOutput` should no longer be used for writing
    /// until a new lock is acquired.
    ///
    /// # Returns
    /// - `Ok(())` if the underlying device unlock syscall succeeds.
    ///
    /// # Errors
    /// Propagates any error returned by `syscall_devices(...)` while unlocking.
    pub fn release(&mut self) -> KernelResult<()> {
        match self.output {
            Usart(_) => syscall_devices(
                crate::DeviceType::Peripheral(self.interface_id.unwrap()),
                crate::SysCallDevicesArgs::Unlock,
                KERNEL_MASTER_ID,
            ),
            Display => syscall_devices(
                crate::DeviceType::Display,
                crate::SysCallDevicesArgs::Unlock,
                KERNEL_MASTER_ID,
            ),
        }
    }
}
