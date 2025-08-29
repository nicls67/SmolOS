use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::usart::Uart;
use embassy_stm32::usart::{Config as UartConfig, Parity, StopBits};
use embassy_stm32::{Peripherals, bind_interrupts, usart};
use hal_interface::{Hal, Interface, InterfaceType};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<embassy_stm32::peripherals::USART1>;
});

pub fn init_interfaces(hal: &mut Hal, peripherals: Peripherals) {
    // USER LEDs
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

    // USART1
    let mut uart_cfg = UartConfig::default();
    uart_cfg.baudrate = 115_200;
    uart_cfg.parity = Parity::ParityNone;
    uart_cfg.stop_bits = StopBits::STOP1;

    hal.add_interface(Interface::new(
        "SERIAL_MAIN",
        InterfaceType::Uart(
            Uart::new(
                peripherals.USART1,
                peripherals.PA10,
                peripherals.PA9,
                Irqs,
                peripherals.DMA2_CH7,
                peripherals.DMA2_CH2,
                uart_cfg,
            )
            .unwrap(),
        ),
    ))
    .unwrap();
}
