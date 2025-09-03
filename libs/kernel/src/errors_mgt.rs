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
