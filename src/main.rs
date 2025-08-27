#![no_std]
#![no_main]

use cortex_m_rt::entry;
use hal_interface::{CoreClkConfig, HalConfig};
use kernel::{BootConfig, Milliseconds};

#[entry]
fn main() -> ! {
    // Start kernel
    kernel::boot(BootConfig {
        sched_period: Milliseconds(1000),
        systick_period: Milliseconds(1),
        hal: HalConfig {
            core_clk_config: CoreClkConfig::Max,
        },
    });

    //let mut led = Output::new(p.PJ13, Level::High, Speed::Low);
    //led.set_low();

    loop {}
}
