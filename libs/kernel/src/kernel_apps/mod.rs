use crate::kernel_apps::led_blink::{LED_BLINK_NAME, LED_BLINK_PERIOD};
use crate::scheduler::AppCall;
use crate::syscall;
use crate::{KernelResult, Syscall};

mod led_blink;
mod term_test;

pub fn initialize_kernel_apps() -> KernelResult<()> {
    syscall(Syscall::AddPeriodicTask(
        LED_BLINK_NAME,
        AppCall::AppNoParam(led_blink::led_blink),
        Some(led_blink::init_led_blink),
        LED_BLINK_PERIOD,
        None,
    ))?;

    syscall(Syscall::AddPeriodicTask(
        term_test::TERM_TEST_NAME,
        AppCall::AppNoParam(term_test::term_test),
        Some(term_test::init_term_test),
        term_test::TERM_TEST_PERIOD,
        None,
    ))?;
    Ok(())
}
