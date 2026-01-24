use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use hal_interface::InterfaceWriteActions;

use crate::{
    DeviceType, KernelResult, SysCallDevicesArgs, SysCallHalActions, syscall_devices, syscall_hal,
};

/// Name of the GPIO interface used as the activity LED.
const K_LED_NAME: &str = "ACT_LED";

/// App/owner identifier used when locking and writing to the LED interface.
static G_LED_APP_ID: AtomicU32 = AtomicU32::new(0);

/// Cached interface ID for the LED GPIO, resolved during [`init_led_blink`].
static G_LED_ID: AtomicUsize = AtomicUsize::new(0);

/// Toggle the LED state once.
///
/// # Errors
/// Returns an error if the underlying HAL syscall fails (e.g., invalid ID,
/// interface not locked for this app, or device unavailable).
pub fn led_blink() -> KernelResult<()> {
    syscall_hal(
        G_LED_ID.load(Ordering::Relaxed),
        SysCallHalActions::Write(InterfaceWriteActions::GpioWrite(
            hal_interface::GpioWriteAction::Toggle,
        )),
        G_LED_APP_ID.load(Ordering::Relaxed),
    )?;

    Ok(())
}

/// Initialize LED blinking support by resolving the interface ID and locking it.
///
/// This function:
/// 1) Queries the HAL for the interface ID corresponding to [`K_LED_NAME`]
/// 2) Stores the ID for later use by [`led_blink`]
/// 3) Attempts to lock the device for the current [`G_LED_APP_ID`]
///
/// # Errors
/// Returns an error if the interface ID cannot be resolved or the device lock
/// cannot be obtained.
pub fn init_led_blink() -> KernelResult<()> {
    // Get LED interface ID
    let mut l_id = 0;
    syscall_hal(0, SysCallHalActions::GetID(K_LED_NAME, &mut l_id), 0)?;
    G_LED_ID.store(l_id, Ordering::Relaxed);

    // Try to get a lock on the interface
    syscall_devices(
        DeviceType::Peripheral(l_id),
        SysCallDevicesArgs::Lock,
        G_LED_APP_ID.load(Ordering::Relaxed),
    )
}

/// Store the app/owner ID used for subsequent LED operations.
pub fn led_blink_id_storage(p_id: u32) {
    G_LED_APP_ID.store(p_id, Ordering::Relaxed);
}
