use crate::kernel_apps::led_blink::{LED_APP_ID, LED_BLINK_NAME, LED_BLINK_PERIOD};
use crate::scheduler::AppCall;
use crate::syscall;
use crate::{KernelResult, Syscall};
use core::sync::atomic::Ordering;

mod led_blink;

pub fn initialize_kernel_apps() -> KernelResult<()> {
    let mut app_id = 0;
    syscall(
        Syscall::AddPeriodicTask(
            LED_BLINK_NAME,
            AppCall::AppNoParam(led_blink::led_blink, None),
            Some(led_blink::init_led_blink),
            LED_BLINK_PERIOD,
            None,
            &mut app_id,
        ),
        0,
    )?;
    LED_APP_ID.store(app_id, Ordering::Relaxed);

    Ok(())
}
