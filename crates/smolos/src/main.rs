#![no_std]
#![no_main]

mod interrupts;

use cortex_m_rt::entry;
use hal_interface::Hal;
use kernel::{BootConfig, KernelTimeData, Mhz, Milliseconds};
use kernel_apps::init_kernel_apps;

#[entry]
fn main() -> ! {
    // Initialize Cortex-M core
    kernel::cortex_init();

    // Start systick
    kernel::init_systick(None);

    // Initialize HAL
    let hal = Hal::new().unwrap();

    // Start kernel
    kernel::boot(BootConfig {
        sched_period: Milliseconds(50),
        kernel_time_data: KernelTimeData {
            core_frequency: Mhz(hal.get_core_clk()),
            systick_period: Milliseconds(1),
        },
        hal,
        system_terminal: "SERIAL_MAIN",
        err_led_name: Some("ERR_LED"),
        display_name: Some("LCD"),
    });

    init_kernel_apps().unwrap();

    #[allow(clippy::empty_loop)]
    loop {}
}
