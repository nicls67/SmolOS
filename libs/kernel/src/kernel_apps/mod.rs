use crate::kernel_apps::led_blink::{LED_BLINK_NAME, LED_BLINK_PERIOD};
use crate::syscall;
use crate::{KernelResult, Syscall};

mod led_blink;
mod term_test;

pub fn initialize_kernel_apps() -> KernelResult<()> {
    syscall(Syscall::AddPeriodicTask(
        LED_BLINK_NAME,
        led_blink::led_blink,
        led_blink::init_led_blink,
        LED_BLINK_PERIOD,
    ))?;

    syscall(Syscall::AddPeriodicTask(
        term_test::TERM_TEST_NAME,
        term_test::term_test,
        term_test::init_term_test,
        term_test::TERM_TEST_PERIOD,
    ))?;
    Ok(())
}
