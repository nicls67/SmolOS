#![no_std]

use kernel::{AppConfig, AppStatus, CallPeriodicity, KernelResult, Milliseconds};

use crate::reboot::K_REBOOT_DELAY;

mod led_blink;
mod reboot;

/// Default kernel apps compiled into the firmware.
///
/// Each entry defines:
/// - the app `name` used for lookup/control,
/// - its scheduling `periodicity`,
/// - the function to execute (`app_fn`),
/// - optional lifecycle hooks (`init_fn`, `end_fn`),
/// - and storage for the assigned app id (`app_id_storage`).
const K_DEFAULT_APPS: [AppConfig; 2] = [
    AppConfig {
        name: "led_blink",
        periodicity: CallPeriodicity::Periodic(Milliseconds(1000)),
        app_fn: led_blink::led_blink,
        init_fn: Some(led_blink::init_led_blink),
        end_fn: None,
        app_status: AppStatus::Stopped,
        id: None,
        app_id_storage: Some(led_blink::led_blink_id_storage),
        param_storage: None,
    },
    AppConfig {
        name: "reboot",
        periodicity: CallPeriodicity::PeriodicUntil(
            Milliseconds(1000),
            Milliseconds((K_REBOOT_DELAY + 1) as u32 * 1000),
        ),
        app_fn: reboot::reboot_periodic,
        init_fn: None,
        end_fn: Some(reboot::reboot_end),
        app_status: AppStatus::Stopped,
        id: None,
        app_id_storage: Some(reboot::reboot_app_id_storage),
        param_storage: None,
    },
];

/// List of default apps that should be started automatically during initialization.
const K_DEFAULT_APPS_START_LIST: [&str; 1] = ["led_blink"];

/// Register default kernel apps and start those included in [`K_DEFAULT_APPS_START_LIST`].
pub fn init_kernel_apps() -> KernelResult<()> {
    for l_app in K_DEFAULT_APPS.iter() {
        kernel::apps().add_app(*l_app)?;

        // Check if the app is in the start list
        if K_DEFAULT_APPS_START_LIST.contains(&l_app.name) {
            kernel::apps().start_app(l_app.name)?;
        }
    }

    Ok(())
}
