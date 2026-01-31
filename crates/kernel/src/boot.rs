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

pub struct BootConfig {
    pub sched_period: Milliseconds,
    pub kernel_time_data: KernelTimeData,
    pub hal: Hal,
    pub system_terminal: &'static str,
    pub err_led_name: Option<&'static str>,
    pub display_name: Option<&'static str>,
}

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
