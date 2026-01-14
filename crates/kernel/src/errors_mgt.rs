//! Error/exception management for the kernel.
//!
//! This module provides:
//! - A `HardFault` exception handler that prints the exception frame over semihosting.
//! - A custom `#[panic_handler]` that prints panic information, waits, then resets the MCU.
//! - An `ErrorsManager` used by the kernel to react to runtime errors by updating an error LED,
//!   printing to the terminal, and interacting with the scheduler (abort/retry and LED blink task).
//!
//! # Error LED behavior
//! - **Fatal**: LED forced ON, then the system panics (and resets via the panic handler).
//! - **Critical**: LED forced ON, message printed, current task aborted.
//! - **Error**: LED blinks for a limited duration (scheduled periodic task), message printed.

use crate::KernelErrorLevel::{Critical, Error, Fatal};
use crate::console_output::ConsoleFormatting;
use crate::console_output::ConsoleFormatting::StrNewLineBoth;
use crate::data::Kernel;
use crate::ident::{KERNEL_MASTER_ID, KERNEL_NAME};
use crate::scheduler::AppCall;
use crate::{
    KernelError, KernelErrorLevel, KernelResult, Milliseconds, SysCallHalActions, syscall_devices,
    syscall_hal, syscall_scheduler,
};
use core::panic::PanicInfo;
use cortex_m_rt::{ExceptionFrame, exception};
use cortex_m_semihosting::hprintln;
use display::Colors;
use hal_interface::{GpioWriteAction, InterfaceWriteActions};

/// Cortex-M HardFault exception handler.
///
/// # Parameters
/// - `ef`: The CPU-provided exception frame captured at the time of the fault.
///
/// # Returns
/// - Never returns (`!`). The handler loops indefinitely after printing the frame.
///
/// # Errors
/// - No recoverable errors are returned. Printing is best-effort via semihosting.
#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    hprintln!("{:#?}", ef);

    #[allow(clippy::empty_loop)]
    loop {}
}

/// Kernel-wide panic handler.
///
/// Prints the kernel name and panic information using semihosting, then waits and resets the MCU.
///
/// # Parameters
/// - `info`: Rust panic payload and location information.
///
/// # Returns
/// - Never returns (`!`). The function resets the system.
///
/// # Errors
/// - No recoverable errors are returned. Output is best-effort via semihosting.
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

/// Centralized manager for kernel error handling.
///
/// Tracks whether an error has occurred and its highest severity, and optionally controls an
/// error LED (via HAL) to reflect error state.
pub struct ErrorsManager {
    /// Optional HAL interface ID for the error LED.
    err_led_id: Option<usize>,
    /// Highest-severity error observed so far (if any).
    has_error: Option<KernelErrorLevel>,
}

impl ErrorsManager {
    /// Name of the periodic scheduler task used to blink the error LED.
    const LED_BLINK_APP_NAME: &'static str = "ERR_LED_BLINK";

    /// Create a new `ErrorsManager` with no configured LED and no recorded errors.
    ///
    /// # Parameters
    /// - None.
    ///
    /// # Returns
    /// - A new `ErrorsManager` instance.
    ///
    /// # Errors
    /// - Does not return errors.
    pub fn new() -> ErrorsManager {
        ErrorsManager {
            err_led_id: None,
            has_error: None,
        }
    }

    /// Initialize the manager and optionally bind to an error LED.
    ///
    /// When `err_led_name` is provided, this function:
    /// 1. Queries the HAL for the interface ID corresponding to the name.
    /// 2. Locks the peripheral so it can be controlled exclusively by the kernel.
    /// 3. Ensures the LED is initially OFF.
    ///
    /// # Parameters
    /// - `err_led_name`: Optional HAL name of the LED interface to use for error indication.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(KernelError)` if HAL ID lookup, device lock, or LED write fails.
    ///
    /// # Errors
    /// - Propagates errors from `syscall_hal` (ID lookup / write) and `syscall_devices` (lock).
    pub fn init(&mut self, err_led_name: Option<&'static str>) -> KernelResult<()> {
        if let Some(name) = err_led_name {
            // Get LED interface ID from HAL
            let mut id = 0;
            syscall_hal(0, SysCallHalActions::GetID(name, &mut id), KERNEL_MASTER_ID)?;
            self.err_led_id = Some(id);

            // Get a lock on the error LED
            syscall_devices(
                crate::DeviceType::Peripheral(self.err_led_id.unwrap()),
                crate::SysCallDevicesArgs::Lock,
                KERNEL_MASTER_ID,
            )?;
        }

        self.set_err_led(false)?;
        Ok(())
    }

    /// Set the error LED state if an LED is configured.
    ///
    /// # Parameters
    /// - `state`: `true` to turn the LED ON, `false` to turn it OFF.
    ///
    /// # Returns
    /// - `Ok(())` if no LED is configured or if the HAL write succeeds.
    /// - `Err(KernelError)` if the HAL write fails.
    ///
    /// # Errors
    /// - Propagates errors from `syscall_hal` when writing to the GPIO interface.
    fn set_err_led(&mut self, state: bool) -> KernelResult<()> {
        if let Some(id) = self.err_led_id {
            syscall_hal(
                id,
                SysCallHalActions::Write(InterfaceWriteActions::GpioWrite(if state {
                    GpioWriteAction::Set
                } else {
                    GpioWriteAction::Clear
                })),
                KERNEL_MASTER_ID,
            )?;
        }
        Ok(())
    }

    /// Handle a `KernelError` by severity and update kernel state accordingly.
    ///
    /// - **Fatal**: Turn LED ON, store severity, then panic (which ultimately resets).
    /// - **Critical**: Turn LED ON, store severity (unless already Fatal), print message, abort
    ///   the currently running task.
    /// - **Error**: Store severity (unless already Critical/Fatal), schedule a temporary LED blink
    ///   task (or extend its duration), clear terminal, print message.
    ///
    /// # Parameters
    /// - `err`: The error to handle.
    ///
    /// # Returns
    /// - This function does not return a `Result`. For `Fatal` errors it does not return at all
    ///   due to `panic!`.
    ///
    /// # Errors
    /// - Internal operations (LED writes, scheduler calls, terminal writes) are best-effort and
    ///   largely ignored via `unwrap_or(())` to avoid recursive failure while handling an error.
    pub fn error_handler(&mut self, err: &KernelError) {
        match err.severity() {
            Fatal => {
                self.set_err_led(true).unwrap_or(());
                self.has_error = Some(Fatal);
                panic!("{}", err.to_string())
            }
            Critical => {
                self.set_err_led(true).unwrap_or(());
                if self.has_error != Some(Fatal) {
                    self.has_error = Some(Critical);
                }
                Kernel::terminal().set_display_mirror(true).unwrap();
                Kernel::terminal().set_color(Colors::Magenta).unwrap();
                Kernel::terminal()
                    .write(&StrNewLineBoth(err.to_string().as_str()))
                    .unwrap_or(());
                Kernel::scheduler().abort_task_on_error();
                Kernel::terminal().set_display_mirror(false).unwrap();
            }
            Error => {
                if self.has_error != Some(Fatal) && self.has_error != Some(Critical) {
                    self.has_error = Some(Error);
                }

                if let Some(id) = self.err_led_id {
                    if Kernel::scheduler()
                        .app_exists(Self::LED_BLINK_APP_NAME, self.err_led_id.map(|x| x as u32))
                        .is_none()
                    {
                        let mut err_app_id = 0;
                        syscall_scheduler(crate::SysCallSchedulerArgs::AddPeriodicTask(
                            Self::LED_BLINK_APP_NAME,
                            AppCall::AppParam(blink_err_led, id as u32),
                            None,
                            Some(reset_err_led),
                            Milliseconds(100),
                            Some(Milliseconds(10000)),
                            &mut err_app_id,
                        ))
                        .unwrap_or(());
                    } else {
                        syscall_scheduler(crate::SysCallSchedulerArgs::NewTaskDuration(
                            Self::LED_BLINK_APP_NAME,
                            Some(id as u32),
                            Milliseconds(10000),
                        ))
                        .unwrap_or(())
                    }
                }

                Kernel::terminal().write(&ConsoleFormatting::Clear).unwrap();
                Kernel::terminal().set_color(Colors::Red).unwrap();
                Kernel::terminal()
                    .write(&StrNewLineBoth(err.to_string().as_str()))
                    .unwrap_or(())
            }
        }
    }

    /// Restore the error LED to match the currently recorded highest-severity error.
    ///
    /// Typically used as a callback after the blink task finishes to ensure the LED ends in the
    /// correct state (OFF for non-critical errors; ON for critical/fatal).
    ///
    /// # Parameters
    /// - None (uses internal state).
    ///
    /// # Returns
    /// - `Ok(())` if no LED is configured or if the HAL write succeeds.
    /// - `Err(KernelError)` if the HAL write fails.
    ///
    /// # Errors
    /// - Propagates errors from `set_err_led` / underlying HAL writes.
    pub(in crate::errors_mgt) fn reset_err_led(&mut self) -> KernelResult<()> {
        if let Some(err_lvl) = self.has_error {
            match err_lvl {
                Error => self.set_err_led(false),
                Critical | Fatal => self.set_err_led(true),
            }
        } else {
            self.set_err_led(false)
        }
    }
}

/// Scheduler task body: toggle the configured error LED.
///
/// Intended to be scheduled periodically to create a blinking pattern.
///
/// # Parameters
/// - `id`: HAL interface ID of the LED to toggle.
///
/// # Returns
/// - `Ok(())` if the toggle write succeeds.
/// - `Err(KernelError)` if the HAL write fails.
///
/// # Errors
/// - Propagates errors from `syscall_hal` when toggling the GPIO.
fn blink_err_led(id: u32) -> KernelResult<()> {
    syscall_hal(
        id as usize,
        SysCallHalActions::Write(InterfaceWriteActions::GpioWrite(GpioWriteAction::Toggle)),
        KERNEL_MASTER_ID,
    )
}

/// Scheduler callback to restore the error LED state after blinking.
///
/// # Parameters
/// - None.
///
/// # Returns
/// - `Ok(())` if the LED state is successfully restored (or no LED is configured).
/// - `Err(KernelError)` if restoring the LED state fails.
///
/// # Errors
/// - Propagates errors from `Kernel::errors().reset_err_led()`.
fn reset_err_led() -> KernelResult<()> {
    Kernel::errors().reset_err_led()
}
