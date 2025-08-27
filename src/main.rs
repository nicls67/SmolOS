#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embassy_stm32::gpio::{Level, Output, Speed};
use hal_interface::{CoreClkConfig, Hal, HalConfig, Interface, InterfaceType};
use kernel::{BootConfig, Milliseconds};

#[entry]
fn main() -> ! {
    // HAL initialization
    let (peripherals, core_freq) = Hal::init(HalConfig {
        core_clk_config: CoreClkConfig::Max,
    });
    let mut hal = Hal::new();

    // Add interfaces
    hal.add_interface(Interface::new(
        "ERR_LED",
        InterfaceType::GpioOutput(Output::new(peripherals.PJ13, Level::High, Speed::Low)),
    ))
    .unwrap();
    hal.add_interface(Interface::new(
        "ACT_LED",
        InterfaceType::GpioOutput(Output::new(peripherals.PJ5, Level::High, Speed::Low)),
    ))
    .unwrap();

    // Lock HAL
    hal.lock().unwrap();

    // Start kernel
    kernel::boot(BootConfig {
        sched_period: Milliseconds(1000),
        systick_period: Milliseconds(1),
        core_freq,
        hal,
    });

    //let mut led = Output::new(p.PJ13, Level::High, Speed::Low);
    //led.set_low();

    loop {}
}
