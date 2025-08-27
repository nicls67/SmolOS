use cortex_m::Peripherals;
use hal_interface::Hal;

use panic_semihosting as _;

/// A mutable static variable holding the global kernel data.
///
/// # Description
/// `KERNEL_DATA` is a mutable static instance of the [`Kernel`] struct.
/// It is initialized with default values, where:
/// - `cortex_peripherals` is set to `None`
/// - `hal` is set to `None`
///
/// This variable is a central point to store global references to core
/// peripherals and hardware abstraction layers (HALs) for the system.
///
/// # Safety
/// Since `KERNEL_DATA` is mutable and static, it introduces the possibility of
/// undefined behavior if misused in a multi-threaded context. Accessing or
/// modifying this variable must be done
pub static mut KERNEL_DATA: Kernel = Kernel {
    cortex_peripherals: None,
    hal: None,
    core_freq: 0,
};

/// The `Kernel` struct represents the core entity responsible for managing
/// hardware peripherals and hardware abstraction layer (HAL) components
/// within the application.
///
/// This structure encapsulates two key optional elements:
/// - `cortex_peripherals`: An optional field representing the peripherals
///   specific to the Cortex microcontroller, typically used for interacting
///   with hardware at a low level. This uses the `Peripherals` type.
/// - `hal`: An optional field encapsulating the hardware abstraction layer,
///   which provides higher-level functionality over the raw hardware peripherals.
///
/// # Fields
/// * `cortex_peripherals` - Optionally holds the `Peripherals` object,
///   enabling direct control over the underlying microcontroller features.
/// * `hal` - Optionally holds the `Hal` object, providing a layer of abstraction
///   for easier access to the microcontroller's features.
///
/// The `Kernel` struct is designed to allow modular and flexible initialization
/// for embedded systems, enabling the user to configure these components as required.
pub struct Kernel {
    cortex_peripherals: Option<Peripherals>,
    hal: Option<Hal>,
    core_freq: u32,
}

impl Kernel {
    /// Initializes the kernel data with the provided hardware abstraction layer (HAL) and stores
    /// the Cortex-M peripherals for further use.
    ///
    /// # Parameters
    /// - `hal`: An instance of the `Hal` structure that represents the hardware abstraction layer to
    ///          initialize the kernel with.
    ///
    /// # Safety
    /// This function involves unsafe operations to directly modify the global `KERNEL_DATA`. The caller
    /// must ensure that this function is invoked in a single-threaded context during system initialization
    /// to prevent data races or undefined behavior.
    ///
    /// # Panics
    /// This function will panic if the Cortex-M peripherals cannot be acquired using `Peripherals::take()`.
    /// Ensure that this function is not called more than once or after the peripherals are already taken.
    ///
    pub fn init_kernel_data(hal: Hal, core_freq: u32) {
        unsafe {
            KERNEL_DATA.cortex_peripherals = Some(Peripherals::take().unwrap());
            KERNEL_DATA.hal = Some(hal);
            KERNEL_DATA.core_freq = core_freq;
        }
    }

    /// Provides a static reference to the `Hal` instance.
    ///
    /// # Returns
    /// A static reference (`&'static`) to the `Hal` object if it's initialized.
    ///
    /// # Panics
    /// This function will panic with the message `"Hal not initialized"` if the `Hal`
    /// instance has not been set in `KERNEL_DATA`.
    ///
    /// # Safety
    /// This function uses unsafe code to access the static mutable `KERNEL_DATA.hal` value.
    /// The unsafe block assumes that access to `KERNEL_DATA.hal` has been properly
    /// synchronized and initialized before calling this function.
    ///
    /// # Allowance
    /// The `#[allow(static_mut_refs)]` attribute is used to suppress the warning for
    /// accessing mutable statics, as this pattern relies on proper internal synchronization
    /// to ensure safety when manipulating `KERNEL_DATA.hal`.
    ///
    /// # Usage
    /// Ensure that the `Hal` instance is initialized in `KERNEL_DATA.hal` before invoking this function:
    ///
    /// If `KERNEL_DATA.hal` is uninitialized, calling this function will result in a panic.
    ///
    #[allow(static_mut_refs)]
    pub fn hal() -> &'static mut Hal {
        unsafe {
            if KERNEL_DATA.hal.is_some() {
                KERNEL_DATA.hal.as_mut().unwrap()
            } else {
                panic!("Hal not initialized");
            }
        }
    }

    /// Retrieves a mutable reference to the Cortex-M peripherals if they have been initialized.
    ///
    /// # Returns
    /// A mutable reference to the `Peripherals` structure that represents Cortex-M peripherals.
    ///
    /// # Panics
    /// This function will panic if the Cortex-M peripherals have not been initialized before calling this function.
    ///
    /// # Safety
    /// This function involves unsafe operations as it accesses mutable static data. The caller must ensure
    /// that this function is used in a thread-safe manner to avoid data races.
    ///
    /// # Features
    /// - The function allows static mutable references by leveraging `#[allow(static_mut_refs)]`, which is
    ///   inherently unsafe. Use with caution in concurrent environments.
    /// - Accessing the peripherals is protected by an `Option`, ensuring that the code only proceeds
    ///   if the peripherals are initialized.
    ///
    #[allow(static_mut_refs)]
    pub fn cortex_peripherals() -> &'static mut Peripherals {
        unsafe {
            if KERNEL_DATA.cortex_peripherals.is_some() {
                KERNEL_DATA.cortex_peripherals.as_mut().unwrap()
            } else {
                panic!("Cortex-M peripherals not initialized");
            }
        }
    }
}
