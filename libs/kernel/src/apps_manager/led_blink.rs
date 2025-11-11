use crate::SysCallDisplayArgs::WriteCharAtCursor;
use crate::{syscall, KernelResult, SysCallHalActions, SysCallHalArgs, Syscall};
use core::sync::atomic::{AtomicU32, AtomicU8, AtomicUsize, Ordering};
use display::Colors;
use hal_interface::GpioWriteAction::Toggle;
use hal_interface::InterfaceWriteActions;

const LED_NAME: &str = "ACT_LED";
static LED_APP_ID: AtomicU32 = AtomicU32::new(0);
static LED_ID: AtomicUsize = AtomicUsize::new(0);
static DOT_COUNTER: AtomicU8 = AtomicU8::new(0);

pub fn led_blink() -> KernelResult<()> {
    syscall(
        Syscall::Hal(SysCallHalArgs {
            id: LED_ID.load(Ordering::Relaxed),
            action: SysCallHalActions::Write(InterfaceWriteActions::GpioWrite(Toggle)),
        }),
        LED_APP_ID.load(Ordering::Relaxed),
    )?;

    match DOT_COUNTER.fetch_add(1, Ordering::Relaxed) {
        0..3 => syscall(
            Syscall::Display(WriteCharAtCursor('.', Some(Colors::White))),
            LED_APP_ID.load(Ordering::Relaxed),
        )?,
        3 => {
            syscall(
                Syscall::Display(WriteCharAtCursor('\r', None)),
                LED_APP_ID.load(Ordering::Relaxed),
            )?;
            syscall(
                Syscall::Display(WriteCharAtCursor(' ', Some(Colors::White))),
                LED_APP_ID.load(Ordering::Relaxed),
            )?
        }
        4 => syscall(
            Syscall::Display(WriteCharAtCursor(' ', Some(Colors::White))),
            LED_APP_ID.load(Ordering::Relaxed),
        )?,
        5 => {
            syscall(
                Syscall::Display(WriteCharAtCursor(' ', Some(Colors::White))),
                LED_APP_ID.load(Ordering::Relaxed),
            )?;
            syscall(
                Syscall::Display(WriteCharAtCursor('\r', None)),
                LED_APP_ID.load(Ordering::Relaxed),
            )?;
            DOT_COUNTER.store(0, Ordering::Relaxed);
        }
        _ => DOT_COUNTER.store(0, Ordering::Relaxed),
    }

    Ok(())
}

pub fn init_led_blink() -> KernelResult<()> {
    // Get LED interface ID
    let mut id = 0;
    syscall(
        Syscall::Hal(SysCallHalArgs {
            id,
            action: SysCallHalActions::GetID(LED_NAME, &mut id),
        }),
        0,
    )?;
    LED_ID.store(id, Ordering::Relaxed);

    // Try to get a lock on the interface
    syscall(
        Syscall::Hal(SysCallHalArgs {
            id,
            action: SysCallHalActions::Lock,
        }),
        LED_APP_ID.load(Ordering::Relaxed),
    )
}

pub fn led_blink_id_storage(id: u32) {
    LED_APP_ID.store(id, Ordering::Relaxed);
}
