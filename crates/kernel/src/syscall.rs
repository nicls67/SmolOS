use crate::console_output::ConsoleFormatting;
use crate::data::Kernel;
use crate::scheduler::{App, AppCall};
use crate::{DeviceType, KernelError, KernelResult, Milliseconds};
use display::Colors;
use hal_interface::{
    InterfaceCallback, InterfaceReadAction, InterfaceReadResult, InterfaceWriteActions,
};

pub enum SysCallHalActions<'a> {
    Write(InterfaceWriteActions<'a>),
    Read(InterfaceReadAction, &'a mut InterfaceReadResult),
    GetID(&'static str, &'a mut usize),
    ConfigureCallback(InterfaceCallback),
}

/// Dispatches a HAL-related syscall to the currently configured HAL implementation.
///
/// This function wraps HAL operations and normalizes error handling by:
/// - Mapping HAL errors into [`KernelError::HalError`]
/// - Invoking the kernel-wide error handler on failure
///
/// # Parameters
/// - `interface_id`: The numeric identifier of the HAL interface to operate on.
/// - `action`: The action to perform against the interface (read/write/lookup/configure).
/// - `caller_id`: The ID of the calling process/app, used for access control/auditing by the HAL.
///
/// # Returns
/// - `Ok(())` if the action succeeds.
/// - `Err(KernelError)` if the HAL operation fails (after the error handler is invoked).
///
/// # Errors
/// - Returns `Err(KernelError::HalError(_))` when:
///   - `interface_write` fails
///   - `interface_read` fails
///   - `get_interface_id` fails
///   - `configure_callback` fails
///
/// In all error cases, `Kernel::errors().error_handler(&err)` is called before returning the error.
///
/// # Side effects
/// - For [`SysCallHalActions::Read`], writes the read result into the provided
///   [`InterfaceReadResult`] via the mutable reference parameter.
/// - For [`SysCallHalActions::GetID`], writes the resolved interface id into the provided `usize`.
pub fn syscall_hal(
    interface_id: usize,
    action: SysCallHalActions,
    caller_id: u32,
) -> KernelResult<()> {
    let result = match action {
        SysCallHalActions::Write(act) => Kernel::hal()
            .interface_write(interface_id, caller_id, act)
            .map_err(KernelError::HalError),
        SysCallHalActions::Read(act, res) => {
            *res = Kernel::hal()
                .interface_read(interface_id, caller_id, act)
                .map_err(KernelError::HalError)?;
            Ok(())
        }
        SysCallHalActions::GetID(name, id) => match Kernel::hal().get_interface_id(name) {
            Ok(hal_id) => {
                *id = hal_id;
                Ok(())
            }
            Err(e) => Err(KernelError::HalError(e)),
        },
        SysCallHalActions::ConfigureCallback(callback) => Kernel::hal()
            .configure_callback(interface_id, caller_id, callback)
            .map_err(KernelError::HalError),
    };

    match result {
        Ok(..) => Ok(()),
        Err(err) => {
            Kernel::errors().error_handler(&err);
            Err(err)
        }
    }
}

pub enum SysCallDisplayArgs<'a> {
    Clear(Colors),
    SetColor(Colors),
    SetFont(display::FontSize),
    SetCursorPos(u16, u16),
    WriteCharAtCursor(char, Option<Colors>),
    WriteChar(char, u16, u16, Option<Colors>),
    WriteStrAtCursor(&'a str, Option<Colors>),
    WriteStr(&'a str, u16, u16, Option<Colors>),
}

/// Dispatches a display-related syscall to the kernel display driver.
///
/// This function enforces that the caller is authorized to use the display device before
/// performing the requested operation. Errors are mapped into [`KernelError::DisplayError`]
/// and routed through the kernel error handler.
///
/// # Parameters
/// - `args`: The display operation to perform (clear, set color/font, set cursor, draw text).
/// - `caller_id`: The ID of the calling process/app. Used to authorize access to the display.
///
/// # Returns
/// - `Ok(())` if authorization and the display operation succeed.
/// - `Err(KernelError)` if authorization fails or the display operation fails.
///
/// # Errors
/// - Returns any error produced by `Kernel::devices().authorize(DeviceType::Display, caller_id)`.
/// - Returns `Err(KernelError::DisplayError(_))` if the underlying display operation fails.
///
/// In all error cases occurring after the match is evaluated, `Kernel::errors().error_handler(&err)`
/// is called before returning the error.
///
/// # Side effects
/// - Writes to the display framebuffer/hardware through `Kernel::display()`.
pub fn syscall_display(args: SysCallDisplayArgs, caller_id: u32) -> KernelResult<()> {
    // Check for device authorization
    Kernel::devices().authorize(DeviceType::Display, caller_id)?;

    let result = match args {
        SysCallDisplayArgs::Clear(color) => Kernel::display().clear(color),
        SysCallDisplayArgs::SetColor(color) => Kernel::display().set_color(color),
        SysCallDisplayArgs::SetFont(font) => Kernel::display().set_font(font),
        SysCallDisplayArgs::SetCursorPos(x, y) => Kernel::display().set_cursor_pos(x, y),
        SysCallDisplayArgs::WriteCharAtCursor(c, color) => {
            Kernel::display().draw_char_at_cursor(c as u8, color)
        }

        SysCallDisplayArgs::WriteChar(c, x, y, color) => {
            Kernel::display().draw_char(c as u8, x, y, color)
        }
        SysCallDisplayArgs::WriteStrAtCursor(str, color) => {
            Kernel::display().draw_string_at_cursor(str, color)
        }
        SysCallDisplayArgs::WriteStr(str, x, y, color) => {
            Kernel::display().draw_string(str, x, y, color)
        }
    }
    .map_err(KernelError::DisplayError);

    match result {
        Ok(..) => Ok(()),
        Err(err) => {
            Kernel::errors().error_handler(&err);
            Err(err)
        }
    }
}

pub enum SysCallSchedulerArgs<'a> {
    AddPeriodicTask(
        &'static str,
        AppCall,
        Option<App>,
        Option<App>,
        Milliseconds,
        Option<Milliseconds>,
        &'a mut u32,
    ),
    RemovePeriodicTask(&'static str, Option<u32>),
    NewTaskDuration(&'static str, Option<u32>, Milliseconds),
}

/// Dispatches scheduler-related syscalls (periodic task creation/removal/configuration).
///
/// This is a thin wrapper around [`Kernel::scheduler()`] methods, and ensures any error
/// is passed through the kernel error handler before being returned.
///
/// # Parameters
/// - `args`: The scheduler operation to perform:
///   - `AddPeriodicTask(name, app, init, period, ends_in, id_out)`
///     - `name`: Task name/identifier.
///     - `app`: The app entry/call to schedule.
///     - `init`: Optional app instance/state to pass to the scheduler.
///     - `period`: The periodic interval in milliseconds.
///     - `ends_in`: Optional duration after which the task should stop.
///     - `id_out`: Output parameter; on success receives the newly created task id.
///   - `RemovePeriodicTask(name, param)`
///     - `name`: Task name/identifier.
///     - `param`: Optional task id to disambiguate/select a specific instance.
///   - `NewTaskDuration(name, param, time)`
///     - `name`: Task name/identifier.
///     - `param`: Optional task id to select a specific instance.
///     - `time`: New duration/limit in milliseconds.
///
/// # Returns
/// - `Ok(())` if the scheduler operation succeeds.
/// - `Err(KernelError)` if the scheduler operation fails.
///
/// # Errors
/// - Propagates any error returned by:
///   - `Kernel::scheduler().add_periodic_app(...)`
///   - `Kernel::scheduler().remove_periodic_app(...)`
///   - `Kernel::scheduler().set_new_task_duration(...)`
///
/// In all error cases, `Kernel::errors().error_handler(&err)` is called before returning the error.
///
/// # Side effects
/// - For `AddPeriodicTask`, writes the created task id into the provided `&mut u32`.
pub fn syscall_scheduler(args: SysCallSchedulerArgs) -> KernelResult<()> {
    let result = match args {
        SysCallSchedulerArgs::AddPeriodicTask(name, app, init, closure, period, ends_in, id) => {
            match Kernel::scheduler().add_periodic_app(name, app, init, closure, period, ends_in) {
                Ok(new_id) => {
                    *id = new_id;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        SysCallSchedulerArgs::RemovePeriodicTask(name, param) => {
            Kernel::scheduler().remove_periodic_app(name, param)
        }
        SysCallSchedulerArgs::NewTaskDuration(name, param, time) => {
            Kernel::scheduler().set_new_task_duration(name, param, time)
        }
    };

    match result {
        Ok(..) => Ok(()),
        Err(err) => {
            Kernel::errors().error_handler(&err);
            Err(err)
        }
    }
}

/// Writes formatted output to the terminal device.
///
/// This function enforces that the caller is authorized to use the terminal device before
/// performing the write. Any write error is routed through the kernel error handler.
///
/// # Parameters
/// - `formatting`: The terminal formatting payload to write (text plus style/format settings).
/// - `caller_id`: The ID of the calling process/app. Used to authorize access to the terminal.
///
/// # Returns
/// - `Ok(())` if authorization and the terminal write succeed.
/// - `Err(KernelError)` if authorization fails or the terminal write fails.
///
/// # Errors
/// - Propagates any error produced by `Kernel::devices().authorize(DeviceType::Terminal, caller_id)`.
/// - Propagates any error returned by `Kernel::terminal().write(&formatting)`.
///
/// In all error cases, `Kernel::errors().error_handler(&err)` is called before returning the error.
///
/// # Side effects
/// - Writes to the terminal output device.
pub fn syscall_terminal(formatting: ConsoleFormatting, caller_id: u32) -> KernelResult<()> {
    // Check for device authorization
    Kernel::devices().authorize(DeviceType::Terminal, caller_id)?;

    match Kernel::terminal().write(&formatting) {
        Ok(..) => Ok(()),
        Err(err) => {
            Kernel::errors().error_handler(&err);
            Err(err)
        }
    }
}

pub enum SysCallDevicesArgs<'a> {
    Lock,
    Unlock,
    GetState(&'a mut bool),
}

/// Dispatches device-management syscalls (lock/unlock/query) for a given device type.
///
/// This function provides a uniform entry point for device locking semantics and state queries.
/// Any underlying error is routed through the kernel error handler.
///
/// # Parameters
/// - `device_type`: The target device type to operate on (e.g. Display, Terminal, etc.).
/// - `args`: The device operation to perform:
///   - `Lock`: Attempt to lock the device for `caller_id`.
///   - `Unlock`: Attempt to unlock the device for `caller_id`.
///   - `GetState(state_out)`: Query whether the device is locked; writes result into `state_out`.
/// - `caller_id`: The ID of the calling process/app, used for ownership checks during lock/unlock.
///
/// # Returns
/// - `Ok(())` if the requested operation succeeds.
/// - `Err(KernelError)` if the operation fails.
///
/// # Errors
/// - Propagates any error returned by:
///   - `Kernel::devices().lock(device_type, caller_id)`
///   - `Kernel::devices().unlock(device_type, caller_id)`
///   - `Kernel::devices().is_locked(device_type)`
///
/// In all error cases, `Kernel::errors().error_handler(&err)` is called before returning the error.
///
/// # Side effects
/// - For `GetState`, writes the locked/unlocked state into the provided `&mut bool`.
pub fn syscall_devices(
    device_type: DeviceType,
    args: SysCallDevicesArgs,
    caller_id: u32,
) -> KernelResult<()> {
    let result = match args {
        SysCallDevicesArgs::Lock => Kernel::devices().lock(device_type, caller_id),
        SysCallDevicesArgs::Unlock => Kernel::devices().unlock(device_type, caller_id),
        SysCallDevicesArgs::GetState(state) => {
            *state = Kernel::devices().is_locked(device_type)?;
            Ok(())
        }
    };

    match result {
        Ok(..) => Ok(()),
        Err(err) => {
            Kernel::errors().error_handler(&err);
            Err(err)
        }
    }
}
