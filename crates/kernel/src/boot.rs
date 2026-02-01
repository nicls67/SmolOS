use crate::apps::AppsManager;
use crate::console_output::ConsoleFormatting;
use crate::data::Kernel;
use crate::devices::DevicesManager;
use crate::errors_mgt::ErrorsManager;
use crate::ident::{K_KERNEL_MASTER_ID, K_KERNEL_NAME, K_KERNEL_VERSION};
use crate::kernel_apps::init_kernel_apps;
use crate::scheduler::Scheduler;
use crate::terminal::Terminal;
use crate::{KernelTimeData, Milliseconds, init_systick};
use display::FontSize::Font24;
use display::{Colors, Display};
use hal_interface::Hal;
use heapless::format;

/// Configuration parameters for the kernel boot process.
pub struct BootConfig {
    /// The scheduling period for the kernel scheduler.
    pub sched_period: Milliseconds,
    /// Timing configuration including core frequency and systick period.
    pub kernel_time_data: KernelTimeData,
    /// The Hardware Abstraction Layer instance.
    pub hal: Hal,
    /// The name of the terminal interface to use for system output.
    pub system_terminal: &'static str,
    /// Optional name of the LED interface to use for error indication.
    pub err_led_name: Option<&'static str>,
    /// Optional name of the display interface to use for system output.
    pub display_name: Option<&'static str>,
}

/// Initializes and starts the kernel.
///
/// This function performs the following steps:
/// 1. Initializes global kernel data (scheduler, hal, terminal, etc.).
/// 2. Configures the HAL locker with the kernel master ID.
/// 3. Initializes the error manager and display.
/// 4. Starts the system terminal and logs boot information.
/// 5. Initializes and starts the SysTick timer.
/// 6. Starts the kernel scheduler.
/// 7. Registers core kernel applications.
///
/// # Parameters
/// - `p_config`: The [`BootConfig`] containing all necessary parameters for booting.
///
/// # Panics
/// This function will panic if any critical initialization step fails (e.g., terminal
/// initialization, display initialization, or scheduler startup).
pub fn boot(p_config: BootConfig) {
    //////////////////////////
    // Kernel initialization
    //////////////////////////
    let l_sched = Scheduler::new(p_config.sched_period);
    Kernel::init_kernel_data(
        p_config.hal,
        Display::new(K_KERNEL_MASTER_ID),
        p_config.kernel_time_data.clone(),
        Terminal::new(p_config.system_terminal).unwrap(),
        l_sched,
        ErrorsManager::new(),
        AppsManager::new(),
        DevicesManager::new(),
    );
    Kernel::hal().configure_locker(K_KERNEL_MASTER_ID).unwrap();

    ////////////////////////////////////
    // Errors Manager initialization
    ////////////////////////////////////
    Kernel::errors().init(p_config.err_led_name).unwrap();

    //////////////////////////
    // Display initialization
    //////////////////////////
    Kernel::display()
        .init(p_config.display_name.unwrap(), Kernel::hal(), Colors::Black)
        .unwrap();
    Kernel::display().set_font(Font24).unwrap();

    ////////////////////////////
    // Terminal start
    ////////////////////////////
    let l_terminal = Kernel::terminal();
    l_terminal.set_display_mode().unwrap();
    l_terminal.set_display_mirror(true).unwrap();
    l_terminal.write(&ConsoleFormatting::Clear).unwrap();
    l_terminal
        .write(&ConsoleFormatting::StrNewLineAfter("Booting..."))
        .unwrap();
    l_terminal
        .write(&ConsoleFormatting::StrNewLineAfter(
            format!(30; "{} version {}", K_KERNEL_NAME, K_KERNEL_VERSION)
                .unwrap()
                .as_str(),
        ))
        .unwrap();
    l_terminal
        .write(&ConsoleFormatting::StrNewLineAfter(
            format!(30; "Core frequency is {} MHz", Kernel::time_data().core_frequency.to_u32() / 1_000_000)
                .unwrap()
                .as_str(),
        ))
        .unwrap();

    ////////////////////////////////////
    // Systick initialization
    ////////////////////////////////////
    init_systick(Some(p_config.kernel_time_data.systick_period));

    //Boot completed
    l_terminal.set_color(Colors::Green).unwrap();
    l_terminal
        .write(&ConsoleFormatting::StrNewLineBoth("Kernel ready !"))
        .unwrap();

    // Start scheduler
    Kernel::scheduler()
        .start(Kernel::time_data().clone().systick_period)
        .unwrap();

    // Set terminal in prompt mode
    l_terminal.set_display_mirror(false).unwrap();
    l_terminal.set_prompt_mode().unwrap();

    // Initialize kernel applications
    init_kernel_apps().unwrap();
}
