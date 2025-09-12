use crate::ident::KERNEL_NAME;
use core::panic::PanicInfo;
use cortex_m_rt::{ExceptionFrame, exception};
use cortex_m_semihosting::hprintln;

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
