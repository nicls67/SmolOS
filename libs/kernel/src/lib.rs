#![no_std]
mod data;
mod except;
mod types;

use crate::data::Kernel;
use crate::except::set_ticks_target;
use cortex_m::peripheral::scb::SystemHandler;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_semihosting::hprintln;
use hal_interface::{CoreClkConfig, Hal, HalConfig};
pub use types::*;

pub struct BootConfig {
    pub sched_period: Milliseconds,
    pub systick_period: Milliseconds,
    pub hal: HalConfig,
}

pub fn boot(config: BootConfig) {
    /////////////////////////
    // HAL initialization
    /////////////////////////
    let hal = Hal::init(config.hal);

    /////////////////////////
    // Kernel initialization
    /////////////////////////
    Kernel::init_kernel_data(hal);

    ////////////////////////////////////
    // Timers initialization
    ////////////////////////////////////

    // Initialize Systick at 1ms
    let cortex_p = Kernel::cortex_peripherals();
    cortex_p.SYST.set_clock_source(SystClkSource::Core);
    cortex_p.SYST.clear_current();
    cortex_p
        .SYST
        .set_reload(Kernel::hal().core_clk_freq * config.systick_period.to_u32() / 1000);
    cortex_p.SYST.enable_interrupt();

    // Initialize scheduler periodic IT
    unsafe {
        cortex_p.SCB.set_priority(SystemHandler::PendSV, 0xFF);
        set_ticks_target(config.sched_period.to_u32() / config.systick_period.to_u32())
    }

    // Start Systick
    cortex_p.SYST.enable_counter();

    hprintln!("Boot OK !");
}
