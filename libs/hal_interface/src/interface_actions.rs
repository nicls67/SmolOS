use embassy_stm32::gpio::Output;

pub enum InterfaceWriteActions {
    GpioWrite(GpioWriteActions),
}

impl InterfaceWriteActions {
    pub fn name(&self) -> &'static str {
        match self {
            InterfaceWriteActions::GpioWrite(_) => "GPIO Write",
        }
    }
}

pub enum GpioWriteActions {
    Set,
    Clear,
    Toggle,
}

impl GpioWriteActions {
    pub fn action(&self, pin: &mut Output) {
        match self {
            GpioWriteActions::Set => pin.set_high(),
            GpioWriteActions::Clear => pin.set_low(),
            GpioWriteActions::Toggle => pin.toggle(),
        }
    }
}
