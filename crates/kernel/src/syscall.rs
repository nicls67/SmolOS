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
    p_interface_id: usize,
    p_action: SysCallHalActions,
    p_caller_id: u32,
) -> KernelResult<()> {
    let l_result = match p_action {
        SysCallHalActions::Write(l_act) => Kernel::hal()
            .interface_write(p_interface_id, p_caller_id, l_act)
            .map_err(KernelError::HalError),
        SysCallHalActions::Read(l_act, l_res) => {
            *l_res = Kernel::hal()
                .interface_read(p_interface_id, p_caller_id, l_act)
                .map_err(KernelError::HalError)?;
            Ok(())
        }
        SysCallHalActions::GetID(l_name, l_id) => match Kernel::hal().get_interface_id(l_name) {
            Ok(l_hal_id) => {
                *l_id = l_hal_id;
                Ok(())
            }
            Err(l_e) => Err(KernelError::HalError(l_e)),
        },
        SysCallHalActions::ConfigureCallback(l_callback) => Kernel::hal()
            .configure_callback(p_interface_id, p_caller_id, l_callback)
            .map_err(KernelError::HalError),
    };

    match l_result {
        Ok(..) => Ok(()),
        Err(l_err) => {
            Kernel::errors().error_handler(&l_err);
            Err(l_err)
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
pub fn syscall_display(p_args: SysCallDisplayArgs, p_caller_id: u32) -> KernelResult<()> {
    // Check for device authorization
    Kernel::devices().authorize(DeviceType::Display, p_caller_id)?;

    let l_result = match p_args {
        SysCallDisplayArgs::Clear(l_color) => Kernel::display().clear(l_color),
        SysCallDisplayArgs::SetColor(l_color) => Kernel::display().set_color(l_color),
        SysCallDisplayArgs::SetFont(l_font) => Kernel::display().set_font(l_font),
        SysCallDisplayArgs::SetCursorPos(l_x, l_y) => Kernel::display().set_cursor_pos(l_x, l_y),
        SysCallDisplayArgs::WriteCharAtCursor(l_c, l_color) => {
            Kernel::display().draw_char_at_cursor(l_c as u8, l_color)
        }

        SysCallDisplayArgs::WriteChar(l_c, l_x, l_y, l_color) => {
            Kernel::display().draw_char(l_c as u8, l_x, l_y, l_color)
        }
        SysCallDisplayArgs::WriteStrAtCursor(l_str, l_color) => {
            Kernel::display().draw_string_at_cursor(l_str, l_color)
        }
        SysCallDisplayArgs::WriteStr(l_str, l_x, l_y, l_color) => {
            Kernel::display().draw_string(l_str, l_x, l_y, l_color)
        }
    }
    .map_err(KernelError::DisplayError);

    match l_result {
        Ok(..) => Ok(()),
        Err(l_err) => {
            Kernel::errors().error_handler(&l_err);
            Err(l_err)
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
///   - `AddPeriodicTask(name, app, init, closure, period, ends_in, id_out)`
///     - `name`: Task name/identifier.
///     - `app`: The app entry/call to schedule (see [`AppCall`]).
///     - `init`: Optional initialization function called once before first execution.
///     - `closure`: Optional cleanup function called when the task's lifetime expires.
///     - `period`: The periodic interval in milliseconds.
///     - `ends_in`: Optional duration after which the task should stop.
///     - `id_out`: Output parameter; on success receives the newly created task id.
///   - `RemovePeriodicTask(name, param)`
///     - `name`: Task name/identifier.
///     - `param`: Optional parameter value to disambiguate tasks with the same name.
///   - `NewTaskDuration(name, param, time)`
///     - `name`: Task name/identifier.
///     - `param`: Optional parameter value to select a specific task instance.
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
pub fn syscall_scheduler(p_args: SysCallSchedulerArgs) -> KernelResult<()> {
    let l_result = match p_args {
        SysCallSchedulerArgs::AddPeriodicTask(
            l_name,
            l_app,
            l_init,
            l_closure,
            l_period,
            l_ends_in,
            l_id,
        ) => {
            match Kernel::scheduler()
                .add_periodic_app(l_name, l_app, l_init, l_closure, l_period, l_ends_in)
            {
                Ok(l_new_id) => {
                    *l_id = l_new_id;
                    Ok(())
                }
                Err(l_e) => Err(l_e),
            }
        }
        SysCallSchedulerArgs::RemovePeriodicTask(l_name, l_param) => {
            Kernel::scheduler().remove_periodic_app(l_name, l_param)
        }
        SysCallSchedulerArgs::NewTaskDuration(l_name, l_param, l_time) => {
            Kernel::scheduler().set_new_task_duration(l_name, l_param, l_time)
        }
    };

    match l_result {
        Ok(..) => Ok(()),
        Err(l_err) => {
            Kernel::errors().error_handler(&l_err);
            Err(l_err)
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
pub fn syscall_terminal(p_formatting: ConsoleFormatting, p_caller_id: u32) -> KernelResult<()> {
    // Check for device authorization
    Kernel::devices().authorize(DeviceType::Terminal, p_caller_id)?;

    match Kernel::terminal().write(&p_formatting) {
        Ok(..) => Ok(()),
        Err(l_err) => {
            Kernel::errors().error_handler(&l_err);
            Err(l_err)
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
    p_device_type: DeviceType,
    p_args: SysCallDevicesArgs,
    p_caller_id: u32,
) -> KernelResult<()> {
    let l_result = match p_args {
        SysCallDevicesArgs::Lock => Kernel::devices().lock(p_device_type, p_caller_id),
        SysCallDevicesArgs::Unlock => Kernel::devices().unlock(p_device_type, p_caller_id),
        SysCallDevicesArgs::GetState(l_state) => {
            *l_state = Kernel::devices().is_locked(p_device_type)?;
            Ok(())
        }
    };

    match l_result {
        Ok(..) => Ok(()),
        Err(l_err) => {
            Kernel::errors().error_handler(&l_err);
            Err(l_err)
        }
    }
}
