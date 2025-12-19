use crate::{syscall_devices, syscall_hal, KernelResult, SysCallDevicesArgs, SysCallHalActions};
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use hal_interface::GpioWriteAction::Toggle;
use hal_interface::InterfaceWriteActions;

/// Name of the GPIO interface used as the activity LED.
const LED_NAME: &str = "ACT_LED";

/// App/owner identifier used when locking and writing to the LED interface.
static LED_APP_ID: AtomicU32 = AtomicU32::new(0);

/// Cached interface ID for the LED GPIO, resolved during [`init_led_blink`].
static LED_ID: AtomicUsize = AtomicUsize::new(0);

/// Toggle the LED state once.
///
/// # Errors
/// Returns an error if the underlying HAL syscall fails (e.g., invalid ID,
/// interface not locked for this app, or device unavailable).
pub fn led_blink() -> KernelResult<()> {
    syscall_hal(
        LED_ID.load(Ordering::Relaxed),
        SysCallHalActions::Write(InterfaceWriteActions::GpioWrite(Toggle)),
        LED_APP_ID.load(Ordering::Relaxed),
    )?;

    Ok(())
}

/// Initialize LED blinking support by resolving the interface ID and locking it.
///
/// This function:
/// 1) Queries the HAL for the interface ID corresponding to [`LED_NAME`]
/// 2) Stores the ID for later use by [`led_blink`]
/// 3) Attempts to lock the device for the current [`LED_APP_ID`]
///
/// # Errors
/// Returns an error if the interface ID cannot be resolved or the device lock
/// cannot be obtained.
pub fn init_led_blink() -> KernelResult<()> {
    // Get LED interface ID
    let mut id = 0;
    syscall_hal(0, SysCallHalActions::GetID(LED_NAME, &mut id), 0)?;
    LED_ID.store(id, Ordering::Relaxed);

    // Try to get a lock on the interface
    syscall_devices(
        crate::DeviceType::Peripheral(id),
        SysCallDevicesArgs::Lock,
        LED_APP_ID.load(Ordering::Relaxed),
    )
}

/// Store the app/owner ID used for subsequent LED operations.
pub fn led_blink_id_storage(id: u32) {
    LED_APP_ID.store(id, Ordering::Relaxed);
}
