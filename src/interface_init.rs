use hal_interface::{Hal, Interface, InterfaceType};

pub fn init_interfaces(hal: &mut Hal) {
    // USER LEDs
    hal.add_interface(Interface::new("ERR_LED", InterfaceType::GpioOutput))
        .unwrap();
    hal.add_interface(Interface::new("ACT_LED", InterfaceType::GpioOutput))
        .unwrap();

    // USART1
    hal.add_interface(Interface::new("SERIAL_MAIN", InterfaceType::Uart))
        .unwrap();
}
