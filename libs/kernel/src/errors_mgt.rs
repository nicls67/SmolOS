use crate::TerminalFormatting::StrNewLineBoth;
use crate::data::Kernel;
use crate::ident::KERNEL_NAME;
use crate::scheduler::AppCall;
use crate::{
    KernelError, KernelErrorLevel, KernelResult, Milliseconds, SysCallHalArgs, Syscall,
    TerminalFormatting, syscall,
};
use core::panic::PanicInfo;
use cortex_m_rt::{ExceptionFrame, exception};
use cortex_m_semihosting::hprintln;
use hal_interface::{GpioWriteActions, InterfaceWriteActions};

/// The HardFault exception handler.
///
/// This function is called when a HardFault exception occurs, which is typically
/// triggered by a serious fault such as accessing an invalid memory address or
/// executing an illegal instruction. It is implemented as an infinite loop to halt
/// the program's execution for debugging or analysis.
///
/// # Parameters
/// - `ef`: A reference to the `ExceptionFrame`, which contains the CPU register
///   state (including program counter, stack pointer, etc.) at the time the fault occurred.
///   This may help with debugging and understanding the cause of the hard fault.
///
/// # Safety
/// This function is marked as `unsafe` because it is directly manipulating low-level
/// hardware or interacting with the runtime in an exceptional state. It should be
/// used with caution as it assumes it is operating within an exceptional, low-level
/// context where normal safety guarantees might not apply.
///
/// The function prints the contents of the `ExceptionFrame` using `hprintln` for
/// debug purposes. Developers can inspect this output to analyze the cause of the
/// fault during runtime.
///
/// # Behavior
/// - Prints the `ExceptionFrame` details in a human-readable format for debugging.
/// - Executes an infinite loop to prevent further execution in the faulted state.
///
/// # Example
/// This function is typically registered as a HardFault handler in embedded systems.
/// It does not return due to the infinite loop, ensuring that the program halts
/// execution completely after encountering the fault.
///
#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    hprintln!("{:#?}", ef);

    #[allow(clippy::empty_loop)]
    loop {}
}

/// The panic handler function, responsible for handling panics in the system.
///
/// When a panic occurs in the program, this function gets invoked.
/// It provides information about the panic, performs any necessary cleanup or
/// debug-related actions, and ensures that the system is reset after a delay.
///
/// # Parameters:
/// - `info`: A reference to a `PanicInfo` object containing details about the panic,
///   such as the location of the panic and an optional panic message.
///
/// # Behavior:
/// 1. Logs the following diagnostic information using `hprintln!`:
///    - A generic panic message along with the name of the kernel (`KERNEL_NAME`).
///    - The contents of the provided `PanicInfo`.
///    - A message indicating that the system will reboot in 5 seconds.
/// 2. Waits for a duration of 5 seconds using `cortex_m::asm::delay`.
/// 3. Resets the system using the `sys_reset` method from the `SCB` peripheral.
///
/// # Notes:
/// - The delay is configured to approximately 5 seconds by assuming a system clock
///   rate of 216 MHz (`216_000_000` cycles per second). Adjust the calculation if the
///   clock frequency changes.
/// - The function never returns (`!` return type).
///
/// # Usage:
/// This function is decorated with the `#[panic_handler]` attribute and is intended to be
/// registered as the global panic handler for the application. Ensure only one such handler
/// exists in your codebase, as multiple panic handlers will result in a compile-time error.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Print the panic message
    hprintln!("{} has panicked !!!!!", KERNEL_NAME);
    hprintln!("{}", info);
    hprintln!("\r\nSystem will reboot in 5 seconds...");

    // Wait for 3 seconds
    cortex_m::asm::delay(216_000_000 * 5);

    // Reset the system
    cortex_m::peripheral::SCB::sys_reset();
}

pub struct ErrorsManager {
    err_led_id: Option<usize>,
}

impl ErrorsManager {
    const LED_BLINK_APP_NAME: &'static str = "ERR_LED_BLINK";

    /// Creates a new instance of `ErrorsManager`.
    ///
    /// # Returns
    ///
    /// A new instance of `ErrorsManager`.
    ///
    pub fn new() -> ErrorsManager {
        ErrorsManager { err_led_id: None }
    }

    /// Initializes the kernel or module instance with an optional error LED identifier.
    ///
    /// # Parameters
    /// - `err_led_name`: An `Option` containing a static string slice representing the name of the error LED.
    ///   - If `Some(name)` is provided, the function will attempt to link the error LED by fetching its interface ID
    ///     from the HAL (Hardware Abstraction Layer).
    ///   - If `None` is passed, the optional `err_led_id` remains unset.
    ///
    /// # Behavior
    /// - If an error LED name is provided, this method:
    ///   1. Resolves the LED's interface ID via the HAL interface.
    ///   2. Assigns the retrieved ID to `err_led_id`.
    /// - Once the optional error LED identifier is processed, the function ensures the error LED is turned off by
    ///   calling `set_err_led(false)`.
    ///
    /// # Returns
    /// - Returns `Ok(())` if the initialization succeeds.
    /// - Returns a `KernelError` if fetching the HAL interface ID or setting the error LED fails.
    ///
    /// # Errors
    /// This function may return the following errors wrapped in a `KernelError`:
    /// - `KernelError::HalError`: If the interface ID retrieval for `err_led_name` fails or if the HAL interaction
    ///   encounters an error.
    /// - Any other errors caused by `set_err_led`.
    ///
    pub fn init(&mut self, err_led_name: Option<&'static str>) -> KernelResult<()> {
        if let Some(name) = err_led_name {
            self.err_led_id = Some(
                Kernel::hal()
                    .get_interface_id(name)
                    .map_err(KernelError::HalError)?,
            );
        }

        self.set_err_led(false)?;
        Ok(())
    }

    /// Sets the state of the error LED.
    ///
    /// This function changes the state of the error LED to the specified value (`true` to set it on, `false` to turn it off).
    /// It uses the hardware abstraction layer (HAL) to perform the operation on the hardware interface associated
    /// with the error LED.
    ///
    /// # Parameters
    /// - `state`: A boolean indicating the desired state of the error LED.
    ///   - `true`: Turns the error LED on.
    ///   - `false`: Turns the error LED off.
    ///
    /// # Returns
    /// - `Ok(())` if the state was successfully set or if no error LED is configured (`self.err_led_id` is `None`).
    /// - `Err(KernelError)` if there was an error interfacing with the hardware abstraction layer (HAL).
    ///
    /// # Errors
    /// Returns an error of type `KernelError::HalError` if the HAL operation to change the LED state fails.
    ///
    /// # Safety
    /// This function assumes that the hardware abstraction layer (HAL) is properly initialized.
    /// Ensure that `self.err_led_id` is correctly configured to avoid any runtime issues.
    fn set_err_led(&mut self, state: bool) -> KernelResult<()> {
        if let Some(id) = self.err_led_id {
            Kernel::hal()
                .interface_write(
                    id,
                    InterfaceWriteActions::GpioWrite(if state {
                        GpioWriteActions::Set
                    } else {
                        GpioWriteActions::Clear
                    }),
                )
                .map_err(KernelError::HalError)?;
        }
        Ok(())
    }

    /// Handles errors within the kernel, performing appropriate actions based on the severity
    /// of the error. This function is designed to ensure the system responds correctly to
    /// different error levels by setting indicators, logging messages, or halting tasks.
    ///
    /// # Parameters
    /// - `err`: A reference to a `KernelError` instance that encapsulates details about the error
    ///   such as its severity and message.
    ///
    /// # Behavior
    /// - **Fatal Errors (`KernelErrorLevel::Fatal`)**:
    ///   - Turns on the error LED indicator. If this fails, it is ignored.
    ///   - Causes the system to panic, displaying the error message, effectively halting the kernel.
    ///
    /// - **Critical Errors (`KernelErrorLevel::Critical`)**:
    ///   - Turns on the error LED indicator. If this fails, it is ignored.
    ///   - Logs the error message to the kernel's terminal output.
    ///   - Aborts the currently running task within the scheduler.
    ///
    /// - **Errors (`KernelErrorLevel::Error`)**:
    ///   - Logs the error message to the kernel's terminal output.
    ///   - No further actions are taken.
    ///
    /// # Panics
    /// - If the error severity is `KernelErrorLevel::Fatal`, the function will cause a panic with
    ///   the error message.
    ///
    /// # Errors
    /// - Any failure when operating on the error LED or writing logs to the terminal is silently
    ///   ignored to ensure the handler does not propagate additional errors.
    pub fn error_handler(&mut self, err: &KernelError) {
        match err.severity() {
            KernelErrorLevel::Fatal => {
                self.set_err_led(true).unwrap_or(());
                panic!("{}", err.to_string())
            }
            KernelErrorLevel::Critical => {
                self.set_err_led(true).unwrap_or(());
                Kernel::terminal()
                    .write(&StrNewLineBoth(err.to_string().as_str()))
                    .unwrap_or(());
                Kernel::scheduler().abort_task_on_error()
            }
            KernelErrorLevel::Error => {
                if let Some(id) = self.err_led_id {
                    if !Kernel::scheduler().app_exists(Self::LED_BLINK_APP_NAME) {
                        syscall(Syscall::AddPeriodicTask(
                            Self::LED_BLINK_APP_NAME,
                            AppCall::AppParam(blink_err_led, id as u32),
                            None,
                            Milliseconds(100),
                            Some(Milliseconds(1000)),
                        ))
                        .unwrap_or(());
                    }
                }
                Kernel::terminal()
                    .write(&StrNewLineBoth(err.to_string().as_str()))
                    .unwrap_or(())
            }
        }
    }
}

fn blink_err_led(id: u32) -> KernelResult<()> {
    syscall(Syscall::Hal(SysCallHalArgs {
        id: id as usize,
        write_action: Some(InterfaceWriteActions::GpioWrite(GpioWriteActions::Toggle)),
        read_action: None,
    }))
}
