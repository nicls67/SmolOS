use crate::KernelResult;
use core::sync::atomic::{AtomicU32, Ordering};

static CMD_APP_ID: AtomicU32 = AtomicU32::new(0);

pub fn cmd_app_id_storage(id: u32) {
    CMD_APP_ID.store(id, Ordering::Relaxed);
}

pub fn reboot() -> KernelResult<()> {
    // Reset the system
    cortex_m::peripheral::SCB::sys_reset();
}
