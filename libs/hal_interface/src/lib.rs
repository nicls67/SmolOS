#![no_std]

mod errors;
mod interfaces;
pub use interfaces::{Interface, InterfaceType};

use crate::HalError::{HalAlreadyLocked, InterfaceInitError};
use crate::HalErrorLevel::Error;
use embassy_stm32::rcc::{
    AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllMul, PllPDiv, PllPreDiv, PllSource, Sysclk,
};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, Peripherals};
pub use errors::*;
use heapless::Vec;

pub enum CoreClkConfig {
    Max,
    Default,
}
pub struct HalConfig {
    pub core_clk_config: CoreClkConfig,
}

pub struct Hal {
    interface: Vec<Interface<'static>, 256>,
    locked: bool,
}

impl Default for Hal {
    fn default() -> Self {
        Self::new()
    }
}

impl Hal {
    /// Initializes the Hardware Abstraction Layer (HAL) with the given configuration.
    ///
    /// This function sets up the peripherals and system clocks based on the provided
    /// `HalConfig` structure. It supports both a default configuration and a configuration
    /// that maximizes the core clock frequency to 216 MHz.
    ///
    /// # Arguments
    ///
    /// * `hal_config` - A `HalConfig` instance containing the desired HAL and clock configurations.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `Peripherals`: The initialized peripherals structure for the system.
    /// * `u32`: The core clock frequency in hertz.
    ///
    /// # Behavior
    ///
    /// - By default, the function assumes a core clock frequency of 16 MHz.
    /// - If the `hal_config.core_clk_config` is set to `CoreClkConfig::Max`, the function:
    ///   - Enables the High-Speed Internal (HSI) oscillator.
    ///   - Configures the High-Speed External (HSE) oscillator with a frequency of 25 MHz.
    ///   - Sets the system clock source to the PLL (Phase Locked Loop) output (PLL1_P).
    ///   - Configures the PLL with a divisor, multiplier, and fractional dividers to
    ///     achieve a core frequency of 216 MHz.
    ///   - Sets appropriate prescalers for the AHB and APB1/APB2 buses.
    ///
    /// # Dependencies
    ///
    /// * The function uses the `embassy_stm32::init` method to complete the hardware initialization
    ///   process using the configured values.
    ///
    /// # Note
    ///
    /// Ensure the `HalConfig` structure is properly instantiated and all required fields are set
    /// before calling this function. The configuration assumes that external components connected
    /// to the microcontroller (e.g., an external clock source) are correctly set up to achieve stable
    /// operation.
    pub fn init(hal_config: HalConfig) -> (Peripherals, u32) {
        // Initialize HAL
        let mut config = Config::default();
        let mut core_freq = 16_000_000;

        if let CoreClkConfig::Max = hal_config.core_clk_config {
            config.rcc.hsi = true;
            config.rcc.hse = Some(Hse {
                freq: Hertz(25_000_000),
                mode: HseMode::Oscillator,
            });
            config.rcc.sys = Sysclk::PLL1_P;
            config.rcc.pll_src = PllSource::HSE;
            config.rcc.pll = Some(Pll {
                prediv: PllPreDiv::DIV25,
                mul: PllMul::MUL432,
                divp: Some(PllPDiv::DIV2),
                divq: None,
                divr: None,
            });
            config.rcc.ahb_pre = AHBPrescaler::DIV1;
            config.rcc.apb1_pre = APBPrescaler::DIV4;
            config.rcc.apb2_pre = APBPrescaler::DIV2;

            core_freq = 216_000_000;
        }

        (embassy_stm32::init(config), core_freq)
    }

    /// Creates and returns a new instance of the struct with default values.
    ///
    /// # Returns
    /// A new instance of the struct with the following default values:
    /// - `interface`: An empty `Vec`
    /// - `locked`: `false`
    ///
    pub fn new() -> Self {
        Self {
            interface: Vec::new(),
            locked: false,
        }
    }

    /// Locks the HAL (Hardware Abstraction Layer) and associates it with the provided peripherals.
    ///
    /// This method ensures exclusive access to the hardware peripherals by marking the HAL as locked.
    /// Once locked, subsequent attempts to lock the HAL will return an error until it is unlocked again.
    ///
    /// # Returns
    /// - `Ok(())` if the HAL is successfully locked and the peripherals are associated.
    /// - `Err(HalAlreadyLocked)` if the HAL is already locked and cannot be locked again.
    ///
    /// # Errors
    /// Returns `HalAlreadyLocked` if an attempt is made to lock the HAL while it is already locked.
    ///
    pub fn lock(&mut self) -> HalResult<()> {
        if self.locked {
            Err(HalAlreadyLocked(Error))
        } else {
            self.locked = true;
            Ok(())
        }
    }

    /// Adds a new `Interface` to the list of interfaces managed by this structure.
    ///
    /// # Arguments
    ///
    /// * `interface` - An `Interface<'a>` instance to be added to the current list of interfaces.
    ///
    /// # Returns
    ///
    /// * `HalResult<()>` - Returns `Ok(())` if the interface is successfully added to the list.
    ///
    /// # Errors
    ///
    /// * Returns `HalAlreadyLocked` error if the structure is in a locked state, indicating no modifications are allowed.
    /// * Returns `InterfaceInitError` if the interface cannot be added because the interfaces list is full.
    ///
    /// # Notes
    ///
    /// * The method first checks if the structure is in a locked state (`self.locked`).
    /// * If locked, it returns an error, preventing modifications.
    /// * Otherwise, it tries to push the provided interface to the `self.interface` list.
    /// * Any failure to add the interface results in an `InterfaceInitError`.
    /// * If successful, returns `Ok(())`.
    pub fn add_interface(&mut self, interface: Interface<'static>) -> HalResult<()> {
        if self.locked {
            Err(HalAlreadyLocked(Error))
        } else {
            self.interface
                .push(interface)
                .map_err(|_| InterfaceInitError(Error, "interfaces list is full"))?;
            Ok(())
        }
    }

    // temporary function
    pub fn invert_pin(&mut self) {
        match &mut self.interface.get_mut(0).unwrap().interface {
            InterfaceType::GpioOutput(pin) => {
                if pin.is_set_high() {
                    pin.set_low();
                } else {
                    pin.set_high();
                }
            }
        }
        match &mut self.interface.get_mut(1).unwrap().interface {
            InterfaceType::GpioOutput(pin) => {
                if pin.is_set_high() {
                    pin.set_low();
                } else {
                    pin.set_high();
                }
            }
        }
    }
}
