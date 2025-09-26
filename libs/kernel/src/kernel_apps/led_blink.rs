use crate::{KernelResult, Milliseconds, SysCallHalArgs, Syscall, syscall};
use core::sync::atomic::{AtomicUsize, Ordering};
use hal_interface::GpioWriteAction::Toggle;
use hal_interface::InterfaceActions;

static LED_ID: AtomicUsize = AtomicUsize::new(0);
const LED_NAME: &str = "ACT_LED";
pub const LED_BLINK_NAME: &str = "LED Blink";
pub const LED_BLINK_PERIOD: Milliseconds = Milliseconds(1000);

pub fn led_blink() -> KernelResult<()> {
    syscall(Syscall::Hal(SysCallHalArgs {
        id: LED_ID.load(Ordering::Relaxed),
        action: InterfaceActions::GpioWrite(Toggle),
    }))
}

pub fn init_led_blink() -> KernelResult<()> {
    let mut id = 0;
    syscall(Syscall::HalGetId(LED_NAME, &mut id))?;
    LED_ID.store(id, Ordering::Relaxed);
    Ok(())
}
