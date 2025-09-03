#![no_std]
#![no_main]

mod interface_init;

use crate::interface_init::init_interfaces;
use cortex_m_rt::entry;
use hal_interface::{CoreClkConfig, Hal, HalConfig};
use kernel::{BootConfig, Milliseconds, TerminalType};

#[entry]
fn main() -> ! {
    // HAL initialization
    let (peripherals, core_freq) = Hal::init(HalConfig {
        core_clk_config: CoreClkConfig::Max,
    });
    let mut hal = Hal::new();

    // Add interfaces
    init_interfaces(&mut hal, peripherals);
    // Lock HAL
    hal.lock().unwrap();

    // Start kernel
    kernel::boot(BootConfig {
        sched_period: Milliseconds(1000),
        systick_period: Milliseconds(1),
        core_freq,
        hal,
        system_terminal_name: "SERIAL_MAIN",
        system_terminal_type: TerminalType::Usart,
    });

    #[allow(clippy::empty_loop)]
    loop {}
}
