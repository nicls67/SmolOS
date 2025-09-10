use crate::data::Kernel;
use crate::{KernelError, KernelResult, Milliseconds};
use core::sync::atomic::{AtomicUsize, Ordering};
use hal_interface::GpioWriteActions::Toggle;
use hal_interface::InterfaceWriteActions;

static LED_ID: AtomicUsize = AtomicUsize::new(0);
const LED_NAME: &str = "ACT_LED";
pub const LED_BLINK_NAME: &str = "LED Blink";
pub const LED_BLINK_PERIOD: Milliseconds = Milliseconds(1000);

pub fn led_blink() -> KernelResult<()> {
    Kernel::hal()
        .interface_write(
            LED_ID.load(Ordering::Relaxed),
            InterfaceWriteActions::GpioWrite(Toggle),
        )
        .map_err(KernelError::HalError)
}

pub fn init_led_blink() -> KernelResult<()> {
    LED_ID.store(
        Kernel::hal()
            .get_interface_id(LED_NAME)
            .map_err(KernelError::HalError)?,
        Ordering::Relaxed,
    );
    Ok(())
}
