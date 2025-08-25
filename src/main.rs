#![no_std]
#![no_main]

use cortex_m_rt::entry;
use kernel::{BootConfig, Milliseconds};

#[entry]
fn main() -> ! {
    // Start kernel
    kernel::boot(BootConfig {
        sched_period: Milliseconds(1000),
    });

    //let mut led = Output::new(p.PJ13, Level::High, Speed::Low);
    //led.set_low();

    loop {}
}
