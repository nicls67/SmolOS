#![no_std]
mod async_block;
mod errors;
mod interface_actions;
mod interfaces;

pub use interface_actions::*;
pub use interfaces::{Interface, InterfaceType};

use crate::HalError::{HalAlreadyLocked, HalNotLocked};
use crate::HalErrorLevel::{Critical, Error, Fatal};
use crate::interfaces::InterfaceVect;
use embassy_stm32::rcc::{
    AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllMul, PllPDiv, PllPreDiv, PllSource, Sysclk,
};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, Peripherals};
pub use errors::*;

pub enum CoreClkConfig {
    Max,
    Default,
}
pub struct HalConfig {
    pub core_clk_config: CoreClkConfig,
}

pub struct Hal {
    interface: InterfaceVect,
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

    /// Creates a new instance of the structure.
    ///
    /// # Returns
    ///
    /// Returns an instance of `Self` initialized with default values:
    /// - `interface`: A new, empty `InterfaceVect`.
    /// - `locked`: A boolean flag set to `false`.
    ///
    pub fn new() -> Self {
        Self {
            interface: InterfaceVect::new(),
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
            Err(HalAlreadyLocked)
        } else {
            self.locked = true;
            Ok(())
        }
    }

    /// Adds a new interface to the current instance of the hardware abstraction layer (HAL).
    ///
    /// # Parameters
    /// - `interface`: The `Interface` object to be added to the HAL instance.
    ///
    /// # Returns
    /// - `Ok(())`: If the interface is successfully added.
    /// - `Err(HalAlreadyLocked)`: If the HAL instance is locked and cannot accept new interfaces.
    ///
    /// # Errors
    /// Returns an error of type `HalAlreadyLocked` if the HAL is in a locked state, indicating
    /// that no changes can be made to the interfaces.
    ///
    /// # Notes
    /// Once the HAL instance
    pub fn add_interface(&mut self, interface: Interface) -> HalResult<()> {
        if self.locked {
            Err(HalAlreadyLocked)
        } else {
            self.interface.add_interface(interface)?;
            Ok(())
        }
    }

    /// Retrieves the interface ID associated with the given name.
    ///
    /// # Parameters
    /// - `name`: A static string slice (`&'static str`) representing the name of the interface
    ///           whose ID is to be retrieved.
    ///
    /// # Returns
    /// - `Ok(usize)`: On success, returns the unique ID (`usize`) associated with the specified interface name.
    /// - `Err(HalNotLocked)`: Returns an error if the system is not in a locked state.
    ///
    /// # Errors
    /// - This function returns a `HalNotLocked` error if the `locked` flag is `false`.
    ///   This indicates that the system state is not secured or locked, and the operation cannot proceed.
    ///
    /// # Preconditions
    /// - The system must be in a locked state (`self.locked == true`) before calling this method to successfully
    ///   retrieve the interface ID.
    ///
    pub fn get_interface_id(&self, name: &'static str) -> HalResult<usize> {
        if !self.locked {
            Err(HalNotLocked)
        } else {
            self.interface.get_interface_id(name)
        }
    }

    /// Writes to an interface using the provided ID and action.
    ///
    /// # Parameters
    /// - `id`: The unique identifier of the interface to write to.
    /// - `action`: The action to be performed on the interface, represented by an `InterfaceWriteActions` value.
    ///
    /// # Returns
    /// - `Ok(())`: If the write operation is successful.
    /// - `Err(HalNotLocked(Critical))`: If the system is not in a locked state when attempting the operation.
    ///
    /// # Errors
    /// This function returns an error if the system is not in a locked state (`self.locked` is `false`).
    ///
    /// # Notes
    /// - The function ensures that write operations are only executed when the system is locked to
    ///   maintain synchronization and safety.
    /// - Delegates the actual write operation to the `self.interface.interface_write` method.
    pub fn interface_write(&mut self, id: usize, action: InterfaceWriteActions) -> HalResult<()> {
        if !self.locked {
            Err(HalNotLocked)
        } else {
            self.interface.interface_write(id, action)
        }
    }

    /// Attempts to perform a read operation on the interface identified by the given `id`
    /// with the specified `action`. The operation is conditional on the locking state of
    /// the system.
    ///
    /// # Parameters
    /// - `id`: The unique identifier for the target interface to perform the read operation on.
    /// - `action`: The specific read action to execute, encapsulated by `InterfaceReadActions`.
    ///
    /// # Returns
    /// - `Ok(())` if the read operation is successfully executed.
    /// - `Err(HalNotLocked(Critical))` if the system is not in a locked state and the operation is denied.
    ///
    /// # Behavior
    /// - If `self.locked` is `false`, the function immediately returns an error with a
    ///   `HalNotLocked(Critical)` variant, indicating that the interface is inaccessible without locking.
    /// - Otherwise, it delegates the read operation to the `interface_read` method of the
    ///   underlying `self.interface` object.
    ///
    /// # Errors
    /// The function may return an error under the following conditions:
    /// - The system is not in a locked state (`self.locked` is `false`).
    /// - Any other error encountered by the `interface_read` method of `self.interface`.
    ///
    pub fn interface_read(&mut self, id: usize, action: InterfaceReadActions) -> HalResult<()> {
        if !self.locked {
            Err(HalNotLocked)
        } else {
            self.interface.interface_read(id, action)
        }
    }
}
