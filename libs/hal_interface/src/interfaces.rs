use embassy_stm32::gpio::Output;

pub enum InterfaceType<'a> {
    GpioOutput(Output<'a>),
}

pub struct Interface<'a> {
    name: &'static str,
    pub(crate) interface: InterfaceType<'a>,
}

impl Interface<'_> {
    pub fn new<'a>(name: &'static str, interface: InterfaceType<'a>) -> Interface<'a> {
        Interface { name, interface }
    }
}
