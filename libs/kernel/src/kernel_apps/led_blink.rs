use crate::SysCallDisplayArgs::WriteCharAtCursor;
use crate::{KernelResult, Milliseconds, SysCallHalArgs, Syscall, syscall};
use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use display::Colors;
use hal_interface::GpioWriteAction::Toggle;
use hal_interface::InterfaceWriteActions;

static LED_ID: AtomicUsize = AtomicUsize::new(0);
const LED_NAME: &str = "ACT_LED";
pub const LED_BLINK_NAME: &str = "LED Blink";
pub const LED_BLINK_PERIOD: Milliseconds = Milliseconds(1000);

static DOT_COUNTER: AtomicU8 = AtomicU8::new(0);

pub fn led_blink() -> KernelResult<()> {
    syscall(Syscall::Hal(SysCallHalArgs {
        id: LED_ID.load(Ordering::Relaxed),
        action: InterfaceWriteActions::GpioWrite(Toggle),
    }))?;

    match DOT_COUNTER.fetch_add(1, Ordering::Relaxed) {
        0..3 => syscall(Syscall::Display(WriteCharAtCursor(
            '.',
            Some(Colors::White),
        )))?,
        3 => {
            syscall(Syscall::Display(WriteCharAtCursor('\r', None)))?;
            syscall(Syscall::Display(WriteCharAtCursor(
                ' ',
                Some(Colors::White),
            )))?
        }
        4 => syscall(Syscall::Display(WriteCharAtCursor(
            ' ',
            Some(Colors::White),
        )))?,
        5 => {
            syscall(Syscall::Display(WriteCharAtCursor(
                ' ',
                Some(Colors::White),
            )))?;
            syscall(Syscall::Display(WriteCharAtCursor('\r', None)))?;
            DOT_COUNTER.store(0, Ordering::Relaxed);
        }
        _ => DOT_COUNTER.store(0, Ordering::Relaxed),
    }

    Ok(())
}

pub fn init_led_blink() -> KernelResult<()> {
    let mut id = 0;
    syscall(Syscall::HalGetId(LED_NAME, &mut id))?;
    LED_ID.store(id, Ordering::Relaxed);
    Ok(())
}
