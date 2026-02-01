use crate::{AppConfig, AppStatus, CallPeriodicity, KernelResult, Milliseconds, apps};

use self::reboot::K_REBOOT_DELAY;

mod app_ctrl;
mod err_gen;
mod led_blink;
mod reboot;

/// Default kernel apps compiled into the firmware.
///
/// Each entry defines:
/// - the app `name` used for lookup/control,
/// - its scheduling `periodicity`,
/// - the function to execute (`app_fn`),
/// - optional lifecycle hooks (`init_fn`, `end_fn`),
/// - and the current status/id fields used by the scheduler.
const K_DEFAULT_APPS: [AppConfig; 4] = [
    AppConfig {
        name: "app_ctrl",
        periodicity: CallPeriodicity::Once,
        app_fn: app_ctrl::app_ctrl,
        init_fn: Some(app_ctrl::app_ctrl_init),
        end_fn: None,
        app_status: AppStatus::Stopped,
        id: None,
    },
    AppConfig {
        name: "led_blink",
        periodicity: CallPeriodicity::Periodic(Milliseconds(1000)),
        app_fn: led_blink::led_blink,
        init_fn: Some(led_blink::init_led_blink),
        end_fn: Some(led_blink::stop_led_blink),
        app_status: AppStatus::Stopped,
        id: None,
    },
    AppConfig {
        name: "reboot",
        periodicity: CallPeriodicity::PeriodicUntil(
            Milliseconds(1000),
            Milliseconds((K_REBOOT_DELAY + 1) as u32 * 1000),
        ),
        app_fn: reboot::reboot_periodic,
        init_fn: Some(reboot::reboot_init),
        end_fn: Some(reboot::reboot_end),
        app_status: AppStatus::Stopped,
        id: None,
    },
    AppConfig {
        name: "err_gen",
        periodicity: CallPeriodicity::Once,
        app_fn: err_gen::err_gen,
        init_fn: Some(err_gen::err_gen_init),
        end_fn: None,
        app_status: AppStatus::Stopped,
        id: None,
    },
];

/// List of default apps that should be started automatically during initialization.
const K_DEFAULT_APPS_START_LIST: [&str; 1] = ["led_blink"];

/// Register default kernel apps and start those included in [`K_DEFAULT_APPS_START_LIST`].
pub fn init_kernel_apps() -> KernelResult<()> {
    for l_app in K_DEFAULT_APPS.iter() {
        apps().add_app(*l_app)?;

        // Check if the app is in the start list
        if K_DEFAULT_APPS_START_LIST.contains(&l_app.name) {
            apps().start_app(l_app.name)?;
        }
    }

    Ok(())
}
