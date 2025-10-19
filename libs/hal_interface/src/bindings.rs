use crate::HalError::{
    IncompatibleAction, InterfaceNotFound, ReadOnlyInterface, WriteOnlyInterface, WrongInterfaceId,
};
use crate::interface_read::InterfaceReadAction;
use crate::{GpioWriteAction, HalResult, InterfaceWriteActions, LcdLayer};

#[repr(u8)]
#[allow(dead_code)]
pub enum HalInterfaceResult {
    OK = 0,
    ErrInterfaceNotFound = 1,
    ErrWrongInterfaceId = 2,
    ErrReadOnlyInterface = 3,
    ErrWriteOnlyInterface = 4,
    ErrIncompatibleAction = 5,
}

impl HalInterfaceResult {
    /// Converts a `HalInterfaceResult` into a `HalResult<()>` by mapping each variant
    /// of the enum into either an `Ok(())` or an appropriate error type.
    ///
    /// # Parameters
    /// - `id`: An optional `usize` representing the identifier of the interface,
    ///   which may be required to generate errors for certain cases.
    /// - `name`: An optional static string slice (`&'static str`) that represents
    ///   the name of the interface. Used in the `ErrInterfaceNotFound` case.
    /// - `action`: An optional `InterfaceActions` enum instance, which represents
    ///   a specific action being performed. This is used when handling
    ///   the `ErrIncompatibleAction` variant.
    ///
    /// # Returns
    /// - `HalResult<()>`: Returns `Ok(())` if the variant is `HalInterfaceResult::OK`.
    ///   For other `HalInterfaceResult` variants, it returns an
    ///   appropriate error wrapped in `HalResult::Err`.
    ///
    /// # Errors
    /// - `InterfaceNotFound`: Returned if the variant is `ErrInterfaceNotFound` and
    ///   the `name` parameter is `Some`.
    /// - `WrongInterfaceId`: Returned if the variant is `ErrWrongInterfaceId` with
    ///   an optional `id`. Defaults to 0 if no `id` is provided.
    /// - `ReadOnlyInterface`: Returned if the variant is `ErrReadOnlyInterface` and
    ///   the `id` is used to fetch the interface name.
    /// - `WriteOnlyInterface`: Returned if the variant is `ErrWriteOnlyInterface` and
    ///   the `id` is used to fetch the interface name.
    /// - `IncompatibleAction`: Returned if the variant is `ErrIncompatibleAction`,
    ///   using the provided action's name and the interface name,
    ///   both of which are derived from the parameters.
    ///
    /// # Panics
    /// - This function will panic if `name` is `None` when `ErrInterfaceNotFound` is encountered.
    /// - It will also panic if `id` is `None` in cases where `id.unwrap()` is called (e.g., for
    ///   `ErrReadOnlyInterface`, `ErrWriteOnlyInterface`, or `ErrIncompatibleAction`).
    /// - If the `interface_name(id.unwrap())` function call fails, it may result in an error.
    ///
    pub fn to_result(
        &self,
        id: Option<usize>,
        name: Option<&'static str>,
        action_write: Option<InterfaceWriteActions>,
        action_read: Option<InterfaceReadAction>,
    ) -> HalResult<()> {
        match self {
            HalInterfaceResult::OK => Ok(()),
            HalInterfaceResult::ErrInterfaceNotFound => Err(InterfaceNotFound(name.unwrap())),
            HalInterfaceResult::ErrWrongInterfaceId => Err(WrongInterfaceId(id.unwrap_or(0))),
            HalInterfaceResult::ErrReadOnlyInterface => {
                Err(ReadOnlyInterface(interface_name(id.unwrap())?))
            }
            HalInterfaceResult::ErrWriteOnlyInterface => {
                Err(WriteOnlyInterface(interface_name(id.unwrap())?))
            }
            HalInterfaceResult::ErrIncompatibleAction => Err(IncompatibleAction(
                {
                    if let Some(action) = action_write {
                        action.name()
                    } else if let Some(action) = action_read {
                        action.name()
                    } else {
                        "Unknown"
                    }
                },
                interface_name(id.unwrap())?,
            )),
        }
    }
}

unsafe extern "C" {
    pub fn hal_init();

    pub fn get_interface_id(name: *const u8, id: *mut u8) -> HalInterfaceResult;

    pub fn get_interface_name(id: u8, name: *mut u8) -> HalInterfaceResult;

    pub fn gpio_write(id: u8, action: GpioWriteAction) -> HalInterfaceResult;

    pub fn usart_write(id: u8, str: *const u8, len: u16) -> HalInterfaceResult;

    pub fn get_core_clk() -> u32;

    pub fn lcd_enable(id: u8, enable: bool) -> HalInterfaceResult;

    pub fn lcd_clear(id: u8, layer: LcdLayer, color: u32) -> HalInterfaceResult;

    pub fn lcd_draw_pixel(
        id: u8,
        layer: LcdLayer,
        x: u16,
        y: u16,
        color: u32,
    ) -> HalInterfaceResult;

    pub fn get_lcd_size(id: u8, x: *mut u16, y: *mut u16) -> HalInterfaceResult;

    pub fn get_fb_address(id: u8, layer: LcdLayer, fb_address: *mut u32) -> HalInterfaceResult;

    pub fn set_fb_address(id: u8, layer: LcdLayer, fb_address: u32) -> HalInterfaceResult;
}

/**
 * Retrieves the name of an interface as a static string, given its unique identifier.
 *
 * # Parameters
 * - `id` (usize): The unique identifier of the interface. Only the lower byte
 *   of this value is used to determine the interface identifier.
 *
 * # Returns
 * - `HalResult<&'static str>`:
 *   - `Ok(&'static str)`: A reference to a static string representing the name of the interface.
 *   - `Err(WrongInterfaceId)`: An error if the ID does not correspond to a valid interface.
 *
 * # Behavior
 * - This function internally calls the `get_interface_name` function.
 * - The retrieved name is stored in a buffer, which is then converted to a static slice.
 * - A static string slice is created from this buffer and returned for valid IDs.
 *
 * # Safety
 * - Uses unsafe code to convert a temporary buffer into a static reference.
 * - Assumes that `get_interface_name` populates the buffer correctly, and its output follows valid UTF-8 encoding.
 * - The caller must ensure correctness of associated operations.
 *
 * # Errors
 * - Returns `Err(WrongInterfaceId)` if `get_interface_name` indicates an invalid interface ID or other failure.
 */
pub fn interface_name(id: usize) -> HalResult<&'static str> {
    let mut name = [0; 32];
    match unsafe { get_interface_name(id as u8, &mut name[0]) } {
        HalInterfaceResult::OK => {
            let static_bytes: &'static [u8] =
                unsafe { core::slice::from_raw_parts(name.as_ptr(), name.len()) };
            let static_str = unsafe { core::str::from_utf8_unchecked(static_bytes) };
            Ok(static_str)
        }
        _ => Err(WrongInterfaceId(id)),
    }
}
