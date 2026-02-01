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

/// Main entry point of the Smolos operating system.
///
/// This function is responsible for:
/// 1. Initializing the Cortex-M core peripherals.
/// 2. Initializing the system tick timer with a default value.
/// 3. Initializing the Hardware Abstraction Layer (HAL).
/// 4. Booting the kernel with a specific configuration.
/// 5. Entering an infinite loop as the kernel takes over execution.
///
/// # Returns
/// This function never returns.
///
/// # Panics
/// Panics if HAL initialization or kernel booting fails.
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
