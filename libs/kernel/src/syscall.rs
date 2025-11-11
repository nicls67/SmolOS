use crate::data::Kernel;
use crate::scheduler::{App, AppCall};
use crate::{KernelError, KernelResult, Milliseconds};
use display::Colors;
use hal_interface::{
    InterfaceCallback, InterfaceReadAction, InterfaceReadResult, InterfaceWriteActions,
};

pub struct SysCallHalArgs<'a> {
    pub id: usize,
    pub action: SysCallHalActions<'a>,
}

pub enum SysCallHalActions<'a> {
    Write(InterfaceWriteActions<'a>),
    Read(InterfaceReadAction, &'a mut InterfaceReadResult),
    Lock,
    Unlock,
    GetID(&'static str, &'a mut usize),
    ConfigureCallback(InterfaceCallback),
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

/// Executes a system call operation based on the specified `syscall_type` and `caller_id`.
///
/// # Parameters
///
/// - `syscall_type`: The type of system call operation to execute, defined in the `Syscall` enum.
/// - `caller_id`: The unique identifier of the calling process or application.
///
/// # Returns
///
/// Returns a `KernelResult<()>`, which is:
/// - `Ok(())` if the system call operation is executed successfully.
/// - `Err(KernelError)` if an error occurs during the execution of the system call. The error is
///   logged via the kernel's error handler.
///
/// # Error Handling
///
/// If an error occurs during the execution of the system call:
/// - The kernel's error handler (`Kernel::errors().error_handler`) is invoked with the specific error.
/// - The function returns the error encapsulated in `Err(KernelError)`.
///
pub fn syscall(syscall_type: Syscall, caller_id: u32) -> KernelResult<()> {
    let result = match syscall_type {
        Syscall::Hal(args) => match args.action {
            SysCallHalActions::Write(act) => Kernel::hal()
                .interface_write(args.id, caller_id, act)
                .map_err(KernelError::HalError),
            SysCallHalActions::Read(act, res) => {
                *res = Kernel::hal()
                    .interface_read(args.id, caller_id, act)
                    .map_err(KernelError::HalError)?;
                Ok(())
            }
            SysCallHalActions::Lock => Kernel::hal()
                .lock_interface(args.id, caller_id)
                .map_err(KernelError::HalError),
            SysCallHalActions::Unlock => Kernel::hal()
                .unlock_interface(args.id, caller_id)
                .map_err(KernelError::HalError),
            SysCallHalActions::GetID(name, id) => match Kernel::hal().get_interface_id(name) {
                Ok(hal_id) => {
                    *id = hal_id;
                    Ok(())
                }
                Err(e) => Err(KernelError::HalError(e)),
            },
            SysCallHalActions::ConfigureCallback(callback) => Kernel::hal()
                .configure_callback(args.id, caller_id, callback)
                .map_err(KernelError::HalError),
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
