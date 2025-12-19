use crate::apps::AppsManager;
use crate::console_output::ConsoleFormatting;
use crate::data::Kernel;
use crate::devices::DevicesManager;
use crate::errors_mgt::ErrorsManager;
use crate::ident::{KERNEL_MASTER_ID, KERNEL_NAME, KERNEL_VERSION};
use crate::kernel_apps::init_kernel_apps;
use crate::scheduler::Scheduler;
use crate::terminal::Terminal;
use crate::{KernelTimeData, Milliseconds, init_systick};
use display::FontSize::Font24;
use display::{Colors, Display};
use hal_interface::Hal;
use heapless::format;

pub struct BootConfig {
    pub sched_period: Milliseconds,
    pub kernel_time_data: KernelTimeData,
    pub hal: Hal,
    pub system_terminal: &'static str,
    pub err_led_name: Option<&'static str>,
    pub display_name: Option<&'static str>,
}

pub fn boot(config: BootConfig) {
    //////////////////////////
    // Kernel initialization
    //////////////////////////
    let sched = Scheduler::new(config.sched_period);
    Kernel::init_kernel_data(
        config.hal,
        Display::new(KERNEL_MASTER_ID),
        config.kernel_time_data.clone(),
        Terminal::new(config.system_terminal).unwrap(),
        sched,
        ErrorsManager::new(),
        AppsManager::new(),
        DevicesManager::new(),
    );
    Kernel::hal().configure_locker(KERNEL_MASTER_ID).unwrap();

    ////////////////////////////////////
    // Errors Manager initialization
    ////////////////////////////////////
    Kernel::errors().init(config.err_led_name).unwrap();

    //////////////////////////
    // Display initialization
    //////////////////////////
    Kernel::display()
        .init(config.display_name.unwrap(), Kernel::hal(), Colors::Black)
        .unwrap();
    Kernel::display().set_font(Font24).unwrap();

    ////////////////////////////
    // Terminal start
    ////////////////////////////
    let terminal = Kernel::terminal();
    terminal.set_display_mode().unwrap();
    terminal.set_display_mirror(true).unwrap();
    terminal.write(&ConsoleFormatting::Clear).unwrap();
    terminal
        .write(&ConsoleFormatting::StrNewLineAfter("Booting..."))
        .unwrap();
    terminal
        .write(&ConsoleFormatting::StrNewLineAfter(
            format!(30; "{} version {}", KERNEL_NAME, KERNEL_VERSION)
                .unwrap()
                .as_str(),
        ))
        .unwrap();
    terminal
        .write(&ConsoleFormatting::StrNewLineAfter(
            format!(30; "Core frequency is {} MHz", Kernel::time_data().core_frequency.to_u32() / 1_000_000)
                .unwrap()
                .as_str(),
        ))
        .unwrap();

    ////////////////////////////////////
    // Systick initialization
    ////////////////////////////////////
    init_systick(Some(config.kernel_time_data.systick_period));

    //Boot completed
    terminal.set_color(Colors::Green).unwrap();
    terminal
        .write(&ConsoleFormatting::StrNewLineBoth("Kernel ready !"))
        .unwrap();

    // Initialize default apps
    init_kernel_apps().unwrap();

    // Start scheduler
    Kernel::scheduler()
        .start(Kernel::time_data().clone().systick_period)
        .unwrap();

    // Set terminal in prompt mode
    terminal.set_display_mirror(false).unwrap();
    terminal.set_prompt_mode().unwrap();
}
