use crate::data::Kernel;
use crate::scheduler::{App, AppCall};
use crate::{KernelError, KernelResult, Milliseconds};
use display::Colors;
use hal_interface::InterfaceWriteActions;

pub struct SysCallHalArgs<'a> {
    pub id: usize,
    pub action: InterfaceWriteActions<'a>,
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

pub enum Syscall<'a> {
    Hal(SysCallHalArgs<'a>),
    HalGetId(&'static str, &'a mut usize),
    AddPeriodicTask(
        &'static str,
        AppCall,
        Option<App>,
        Milliseconds,
        Option<Milliseconds>,
        &'a mut u32,
    ),
    RemovePeriodicTask(&'static str, Option<u32>),
    NewTaskDuration(&'static str, Option<u32>, Milliseconds),
    Display(SysCallDisplayArgs<'a>),
}

/// Handles system calls to interact with various kernel subsystems like HAL, Scheduler, and Display.
///
/// # Parameters
/// - `syscall_type`: Specifies the type of system call being requested. It includes various operations supported by the kernel (e.g., interacting with HAL, Scheduler, or Display).
/// - `caller_id`: The identifier of the entity making the system call, often used for permissions or tracking purposes.
///
/// # Returns
/// - `KernelResult<()>`: Returns `Ok(())` if the system call completes successfully.
///   Returns an error wrapped in `KernelResult` if the system call encounters an issue.
///
/// # Supported System Calls
///
/// ## HAL (Hardware Abstraction Layer)
/// - `Syscall::Hal(args)`:
///   Executes an action on a HAL interface identified by `args.id`.
/// - `Syscall::HalGetId(name, id)`:
///   Retrieves the ID of a HAL interface by its `name` and sets its value in `id`.
///
/// ## Scheduler
/// - `Syscall::AddPeriodicTask(name, app, init, period, ends_in, id)`:
///   Adds a periodic task to the scheduler.
///   Assigns a new ID for the task in `id`.
/// - `Syscall::RemovePeriodicTask(name, param)`:
///   Removes a periodic task identified by `name` and optionally `param`.
/// - `Syscall::NewTaskDuration(name, param, time)`:
///   Updates the duration of an existing task in the scheduler.
///
/// ## Display
/// Handles various graphical display operations:
/// - `SysCallDisplayArgs::Clear(color)`:
///   Clears the screen with the specified `color`.
/// - `SysCallDisplayArgs::SetColor(color)`:
///   Sets the display's active color.
/// - `SysCallDisplayArgs::SetFont(font)`:
///   Updates the display's font.
/// - `SysCallDisplayArgs::SetCursorPos(x, y)`:
///   Moves the display's cursor to the specified coordinates (`x`, `y`).
/// - `SysCallDisplayArgs::WriteCharAtCursor(c, color)`:
///   Writes a single character `c` with `color` at the current cursor position.
/// - `SysCallDisplayArgs::WriteChar(c, x, y, color)`:
///   Writes a character `c` at coordinates (`x`, `y`) with the specified `color`.
/// - `SysCallDisplayArgs::WriteStrAtCursor(str, color)`:
///   Writes a string `str` starting at the current cursor position with the specified `color`.
/// - `SysCallDisplayArgs::WriteStr(str, x, y, color)`:
///   Writes a string `str` at coordinates (`x`, `y`) with the specified `color`.
///
/// # Error Handling
/// If any subsystem operation fails, the appropriate error is logged using the kernel's error handler via `Kernel::errors().error_handler(&err)`.
///
pub fn syscall(syscall_type: Syscall, caller_id: u32) -> KernelResult<()> {
    let result = match syscall_type {
        Syscall::Hal(args) => Kernel::hal()
            .interface_write(args.id, caller_id, args.action)
            .map_err(KernelError::HalError),
        Syscall::HalGetId(name, id) => match Kernel::hal().get_interface_id(name) {
            Ok(hal_id) => {
                *id = hal_id;
                Ok(())
            }
            Err(e) => Err(KernelError::HalError(e)),
        },
        Syscall::AddPeriodicTask(name, app, init, period, ends_in, id) => {
            match Kernel::scheduler().add_periodic_app(name, app, init, period, ends_in) {
                Ok(new_id) => {
                    *id = new_id;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        Syscall::RemovePeriodicTask(name, param) => {
            Kernel::scheduler().remove_periodic_app(name, param)
        }
        Syscall::NewTaskDuration(name, param, time) => {
            Kernel::scheduler().set_new_task_duration(name, param, time)
        }
        Syscall::Display(args) => match args {
            SysCallDisplayArgs::Clear(color) => Kernel::display().clear(color, caller_id),
            SysCallDisplayArgs::SetColor(color) => Kernel::display().set_color(color, caller_id),
            SysCallDisplayArgs::SetFont(font) => Kernel::display().set_font(font, caller_id),
            SysCallDisplayArgs::SetCursorPos(x, y) => {
                Kernel::display().set_cursor_pos(x, y, caller_id)
            }
            SysCallDisplayArgs::WriteCharAtCursor(c, color) => {
                Kernel::display().draw_char_at_cursor(c as u8, color, caller_id)
            }

            SysCallDisplayArgs::WriteChar(c, x, y, color) => {
                Kernel::display().draw_char(c as u8, x, y, color, caller_id)
            }
            SysCallDisplayArgs::WriteStrAtCursor(str, color) => {
                Kernel::display().draw_string_at_cursor(str, color, caller_id)
            }
            SysCallDisplayArgs::WriteStr(str, x, y, color) => {
                Kernel::display().draw_string(str, x, y, color, caller_id)
            }
        }
        .map_err(KernelError::DisplayError),
    };

    match result {
        Ok(..) => Ok(()),
        Err(err) => {
            Kernel::errors().error_handler(&err);
            Err(err)
        }
    }
}
