use core::sync::atomic::{AtomicU8, AtomicU32, Ordering};

use heapless::{String, Vec, format};

use crate::{
    ConsoleFormatting, K_MAX_APP_PARAM_SIZE, K_MAX_APP_PARAMS, KernelResult, syscall_terminal,
};

/// Stores the app ID associated with the current command context.
///
/// This ID is used when sending output to the terminal so the message is routed
/// to the correct application session.
static G_REBOOT_APP_ID: AtomicU32 = AtomicU32::new(0);

/// Initialize the reboot app by storing its scheduler id.
///
/// # Parameters
/// - `app_id`: Scheduler id assigned to this app.
/// - `param`: Parsed parameters (unused).
pub fn reboot_init(
    p_app_id: u32,
    _p_param: Vec<String<K_MAX_APP_PARAM_SIZE>, K_MAX_APP_PARAMS>,
) -> KernelResult<()> {
    G_REBOOT_APP_ID.store(p_app_id, Ordering::Relaxed);
    Ok(())
}

/// Perform the final reboot action by resetting the system.
///
/// # Returns
/// This function does not return, as it triggers a system reset.
///
/// # Errors
/// This function never returns an error because the system reset is invoked
/// unconditionally.
pub fn reboot_end() -> KernelResult<()> {
    // Reset the system
    cortex_m::peripheral::SCB::sys_reset();
}

/// Default number of seconds to wait before rebooting.
pub const K_REBOOT_DELAY: u8 = 3;

/// Countdown value used by [`reboot_periodic`] to report remaining time.
static G_REBOOT_COUNTER: AtomicU8 = AtomicU8::new(K_REBOOT_DELAY);

/// Periodic reboot countdown handler.
///
/// Decrements the internal reboot counter and prints a message indicating the
/// remaining time until reboot.
///
/// # Errors
/// Returns any error produced by the terminal syscall.
pub fn reboot_periodic() -> KernelResult<()> {
    syscall_terminal(
        ConsoleFormatting::StrNewLineBefore(
            format!(
                50;
                "Rebooting in {} seconds...",
                G_REBOOT_COUNTER.fetch_sub(1, Ordering::Relaxed)
            )
            .unwrap()
            .as_str(),
        ),
        G_REBOOT_APP_ID.load(Ordering::Relaxed),
    )
}
