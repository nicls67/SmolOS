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
    ),
    RemovePeriodicTask(&'static str, Option<u32>),
    NewTaskDuration(&'static str, Option<u32>, Milliseconds),
    Display(SysCallDisplayArgs<'a>),
}

/**
 * Executes various system calls based on the provided `Syscall` enum variant.
 * This function acts as the central interface for managing kernel operations
 * such as hardware interactions, periodic task scheduling, and modifying task properties.
 *
 * # Arguments
 *
 * * `syscall_type` - A variant of the `Syscall` enum that specifies the type of system call to execute.
 *
 * # Returns
 *
 * * `Ok(())` - If the requested system call is successful.
 * * `Err(KernelError)` - If an error occurs during the execution of the system call, which is then handled
 *   by the kernel's error handler.
 *
 * # Error Handling
 *
 * In the case of an error during the execution of the system call, the kernel error
 * handler processes the error by invoking `Kernel::errors().error_handler()`. The error
 * is then returned as a `KernelError`.
 *
 */
pub fn syscall(syscall_type: Syscall) -> KernelResult<()> {
    let result = match syscall_type {
        Syscall::Hal(args) => Kernel::hal()
            .interface_write(args.id, args.action)
            .map_err(KernelError::HalError),
        Syscall::HalGetId(name, id) => match Kernel::hal().get_interface_id(name) {
            Ok(hal_id) => {
                *id = hal_id;
                Ok(())
            }
            Err(e) => Err(KernelError::HalError(e)),
        },
        Syscall::AddPeriodicTask(name, app, init, period, ends_in) => {
            Kernel::scheduler().add_periodic_app(name, app, init, period, ends_in)
        }
        Syscall::RemovePeriodicTask(name, param) => {
            Kernel::scheduler().remove_periodic_app(name, param)
        }
        Syscall::NewTaskDuration(name, param, time) => {
            Kernel::scheduler().set_new_task_duration(name, param, time)
        }
        Syscall::Display(args) => match args {
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
