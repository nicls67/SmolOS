use crate::KernelResult;
use crate::data::Kernel;
use crate::kernel_apps::led_blink::{LED_BLINK_NAME, LED_BLINK_PERIOD};

mod led_blink;

pub fn initialize_kernel_apps() -> KernelResult<()> {
    Kernel::scheduler().add_periodic_app(
        LED_BLINK_NAME,
        led_blink::led_blink,
        led_blink::init_led_blink,
        LED_BLINK_PERIOD,
    )?;

    Ok(())
}
