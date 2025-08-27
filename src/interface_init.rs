use embassy_stm32::Peripherals;
use embassy_stm32::gpio::{Level, Output, Speed};
use hal_interface::{Hal, Interface, InterfaceType};

pub fn init_interfaces(hal: &mut Hal, peripherals: Peripherals) {
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
}
