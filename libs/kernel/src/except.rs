use crate::data::Kernel;
use core::sync::atomic::{AtomicU32, Ordering, compiler_fence, fence};
use cortex_m::asm::{dsb, isb};
use cortex_m::peripheral::SCB;
use cortex_m::peripheral::scb::VectActive::Exception;
use cortex_m_rt::exception;

static SCHED_TICKS_COUNTER: AtomicU32 = AtomicU32::new(0);
static SCHED_TICKS_TARGET: AtomicU32 = AtomicU32::new(0);

pub fn set_ticks_target(target: u32) {
    SCHED_TICKS_TARGET.store(target, Ordering::Relaxed);
}

#[exception]
fn SysTick() {
    let value = SCHED_TICKS_COUNTER.load(Ordering::Relaxed);

    if value >= SCHED_TICKS_TARGET.load(Ordering::Relaxed) {
        SCB::set_pendsv();
        SCHED_TICKS_COUNTER.store(0, Ordering::Relaxed);
    } else {
        SCHED_TICKS_COUNTER.fetch_add(1, Ordering::Relaxed);
    }
}

#[exception]
fn PendSV() {
    Kernel::scheduler().periodic_task();
}

pub fn return_from_exception() {
    if let Exception(_) = SCB::vect_active() {
        let mut exc: u32;
        unsafe {
            core::arch::asm!("mov {r}, lr", r = out(reg) exc, options(nomem, nostack, preserves_flags));
        }

        compiler_fence(Ordering::SeqCst);
        fence(Ordering::SeqCst);

        dsb();
        isb();

        unsafe {
            core::arch::asm!(
            "mov lr, {exc}",
            "bx lr",
            exc = in(reg) exc,
            options(noreturn)
            );
        }
    }
}
