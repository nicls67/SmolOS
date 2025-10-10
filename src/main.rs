#![no_std]
#![no_main]

use cortex_m_rt::entry;
use hal_interface::Hal;
use heapless::Vec;
use kernel::{BootConfig, KernelTimeData, Mhz, Milliseconds, TerminalType};

#[entry]
fn main() -> ! {
    // Initialize Cortex-M core
    kernel::cortex_init();

    // Start systick
    kernel::init_systick(None);

    // Initialize HAL
    let hal = Hal::new();

    // Start kernel
    kernel::boot(BootConfig {
        sched_period: Milliseconds(50),
        kernel_time_data: KernelTimeData {
            core_frequency: Mhz(hal.get_core_clk()),
            systick_period: Milliseconds(1),
        },
        hal,
        system_terminals: Vec::from_slice(&[
            TerminalType::Usart("SERIAL_MAIN"),
            TerminalType::Display,
        ])
        .unwrap(),
        err_led_name: Some("ERR_LED"),
        display_name: Some("LCD"),
    });

    kernel::start_kernel_apps();
    kernel::start_scheduler();

    #[allow(clippy::empty_loop)]
    loop {}
}
