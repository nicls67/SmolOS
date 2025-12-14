use crate::{KernelResult, SysCallDevicesArgs, SysCallHalActions, syscall_devices, syscall_hal};
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use hal_interface::GpioWriteAction::Toggle;
use hal_interface::InterfaceWriteActions;

const LED_NAME: &str = "ACT_LED";
static LED_APP_ID: AtomicU32 = AtomicU32::new(0);
static LED_ID: AtomicUsize = AtomicUsize::new(0);

pub fn led_blink() -> KernelResult<()> {
    syscall_hal(
        LED_ID.load(Ordering::Relaxed),
        SysCallHalActions::Write(InterfaceWriteActions::GpioWrite(Toggle)),
        LED_APP_ID.load(Ordering::Relaxed),
    )?;

    Ok(())
}

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

pub fn led_blink_id_storage(id: u32) {
    LED_APP_ID.store(id, Ordering::Relaxed);
}
