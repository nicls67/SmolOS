#![no_std]
#![no_main]

//! Smolos Application Entry Point
//!
//! This module contains the main entry point for the Smolos application.
//! It initializes the hardware, kernel, and starts the system.

mod interrupts;

use cortex_m_rt::entry;
use hal_interface::Hal;
use kernel::{BootConfig, KernelTimeData, Mhz, Milliseconds};

#[entry]
fn main() -> ! {
    // Initialize Cortex-M core
    kernel::cortex_init();

    // Start systick
    kernel::init_systick(None);

    // Initialize HAL
    let l_hal = Hal::new().unwrap();

    // Start kernel
    kernel::boot(BootConfig {
        sched_period: Milliseconds(50),
        kernel_time_data: KernelTimeData {
            core_frequency: Mhz(l_hal.get_core_clk()),
            systick_period: Milliseconds(1),
        },
        hal: l_hal,
        system_terminal: "SERIAL_MAIN",
        err_led_name: Some("ERR_LED"),
        display_name: Some("LCD"),
    });

    #[allow(clippy::empty_loop)]
    loop {}
}
