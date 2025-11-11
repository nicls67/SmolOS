use crate::Syscall::TerminalWrite;
use crate::terminal::TerminalFormatting;
use crate::{KernelResult, syscall};
use core::sync::atomic::{AtomicU8, AtomicU32, Ordering};
use heapless::format;

static CMD_APP_ID: AtomicU32 = AtomicU32::new(0);

pub fn cmd_app_id_storage(id: u32) {
    CMD_APP_ID.store(id, Ordering::Relaxed);
}

pub fn reboot_end() -> KernelResult<()> {
    // Reset the system
    cortex_m::peripheral::SCB::sys_reset();
}

pub const REBOOT_DELAY: u8 = 3;
static REBOOT_COUNTER: AtomicU8 = AtomicU8::new(REBOOT_DELAY);

pub fn reboot_periodic() -> KernelResult<()> {
    syscall(TerminalWrite(TerminalFormatting::StrNewLineBefore(
        format!(50; "Rebooting in {} seconds...", REBOOT_COUNTER.fetch_sub(1,Ordering::Relaxed))
            .unwrap()
            .as_str(),
    )),CMD_APP_ID.load(Ordering::Relaxed))
}
