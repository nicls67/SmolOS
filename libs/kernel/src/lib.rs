#![no_std]
mod data;
mod except;
mod types;

use crate::data::Kernel;
use crate::except::set_ticks_target;
use cortex_m::peripheral::scb::SystemHandler;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_semihosting::hprintln;
use hal_interface::Hal;
pub use types::*;

pub struct BootConfig {
    pub sched_period: Milliseconds,
    pub systick_period: Milliseconds,
    pub core_freq: u32,
    pub hal: Hal,
}

pub fn boot(config: BootConfig) {
    /////////////////////////
    // Kernel initialization
    /////////////////////////
    Kernel::init_kernel_data(config.hal, config.core_freq);

    ////////////////////////////////////
    // Timers initialization
    ////////////////////////////////////

    // Initialize Systick at 1ms
    let cortex_p = Kernel::cortex_peripherals();
    cortex_p.SYST.set_clock_source(SystClkSource::Core);
    cortex_p.SYST.clear_current();
    cortex_p
        .SYST
        .set_reload(config.core_freq * config.systick_period.to_u32() / 1000);
    cortex_p.SYST.enable_interrupt();

    // Initialize scheduler periodic IT
    unsafe {
        cortex_p.SCB.set_priority(SystemHandler::PendSV, 0xFF);
        set_ticks_target(config.sched_period.to_u32() / config.systick_period.to_u32())
    }

    // Start Systick
    cortex_p.SYST.enable_counter();
}
