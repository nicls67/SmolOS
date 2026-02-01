use crate::HalError::{
    IncompatibleAction, InterfaceNotFound, ReadOnlyInterface, WriteOnlyInterface, WrongInterfaceId,
};
use crate::interface_read::InterfaceReadAction;
use crate::{
    GpioWriteAction, HalError, HalResult, InterfaceCallback, InterfaceWriteActions, LcdLayer,
    RxBuffer,
};

/// Represents the result codes returned by the underlying C HAL.
#[repr(u8)]
#[allow(dead_code)]
pub enum HalInterfaceResult {
    /// Operation successful.
    OK = 0,
    /// The specified interface was not found.
    ErrInterfaceNotFound = 1,
    /// The provided interface ID is invalid.
    ErrWrongInterfaceId = 2,
    /// Attempted to write to a read-only interface.
    ErrReadOnlyInterface = 3,
    /// Attempted to read from a write-only interface.
    ErrWriteOnlyInterface = 4,
    /// The requested action is not compatible with the interface type.
    ErrIncompatibleAction = 5,
    /// An error occurred during a write operation.
    ErrWriteError = 6,
    /// No buffer is associated with the interface for reading.
    ErrNoBuffer = 7,
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
        p_id: Option<usize>,
        p_name: Option<&'static str>,
        p_action_write: Option<InterfaceWriteActions>,
        p_action_read: Option<InterfaceReadAction>,
    ) -> HalResult<()> {
        match self {
            HalInterfaceResult::OK => Ok(()),
            HalInterfaceResult::ErrInterfaceNotFound => Err(InterfaceNotFound(p_name.unwrap())),
            HalInterfaceResult::ErrWrongInterfaceId => Err(WrongInterfaceId(p_id.unwrap_or(0))),
            HalInterfaceResult::ErrReadOnlyInterface => {
                Err(ReadOnlyInterface(interface_name(p_id.unwrap())?))
            }
            HalInterfaceResult::ErrWriteOnlyInterface => {
                Err(WriteOnlyInterface(interface_name(p_id.unwrap())?))
            }

            HalInterfaceResult::ErrIncompatibleAction => Err(IncompatibleAction(
                {
                    if let Some(l_action) = p_action_write {
                        l_action.name()
                    } else if let Some(l_action) = p_action_read {
                        l_action.name()
                    } else {
                        "Unknown"
                    }
                },
                interface_name(p_id.unwrap())?,
            )),
            HalInterfaceResult::ErrWriteError => {
                Err(HalError::WriteError(interface_name(p_id.unwrap())?))
            }
            HalInterfaceResult::ErrNoBuffer => Err(HalError::InterfaceBadConfig(
                interface_name(p_id.unwrap())?,
                "No buffer provided for read operation",
            )),
        }
    }
}

unsafe extern "C" {
    pub fn hal_init();

    pub fn get_interface_id(p_name: *const u8, p_id: *mut u8) -> HalInterfaceResult;

    pub fn get_interface_name(p_id: u8, p_name: *mut u8) -> HalInterfaceResult;

    pub fn configure_callback(p_id: u8, p_callback: InterfaceCallback) -> HalInterfaceResult;

    pub fn gpio_write(p_id: u8, p_action: GpioWriteAction) -> HalInterfaceResult;

    pub fn usart_write(p_id: u8, p_str: *const u8, p_len: u16) -> HalInterfaceResult;

    pub fn get_read_buffer(p_id: u8, p_buffer: &mut &mut RxBuffer) -> HalInterfaceResult;

    pub fn get_core_clk() -> u32;

    pub fn lcd_enable(p_id: u8, p_enable: bool) -> HalInterfaceResult;

    pub fn lcd_clear(p_id: u8, p_layer: LcdLayer, p_color: u32) -> HalInterfaceResult;

    pub fn lcd_draw_pixel(
        p_id: u8,
        p_layer: LcdLayer,
        p_x: u16,
        p_y: u16,
        p_color: u32,
    ) -> HalInterfaceResult;

    pub fn get_lcd_size(p_id: u8, p_x: *mut u16, p_y: *mut u16) -> HalInterfaceResult;

    pub fn get_fb_address(
        p_id: u8,
        p_layer: LcdLayer,
        p_fb_address: *mut u32,
    ) -> HalInterfaceResult;

    pub fn set_fb_address(p_id: u8, p_layer: LcdLayer, p_fb_address: u32) -> HalInterfaceResult;
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
 * - The retrieved name is stored in a static buffer, trimmed at the first `0` byte,
 *   and returned as a string slice.
 *
 * # Safety
 * - Uses a shared static buffer; repeated calls overwrite previous results.
 * - Assumes that `get_interface_name` populates the buffer correctly, and its output follows valid UTF-8 encoding.
 * - The caller must ensure correctness of associated operations.
 *
 * # Errors
 * - Returns `Err(WrongInterfaceId)` if `get_interface_name` indicates an invalid interface ID or other failure.
 */
pub fn interface_name(p_id: usize) -> HalResult<&'static str> {
    const K_INTERFACE_NAME_BUF_LEN: usize = 32;
    static mut G_INTERFACE_NAME_BUF: [u8; K_INTERFACE_NAME_BUF_LEN] = [0; K_INTERFACE_NAME_BUF_LEN];

    // Ensure trailing bytes are cleared so we can safely trim to content length.
    unsafe {
        let l_buf_ptr = core::ptr::addr_of_mut!(G_INTERFACE_NAME_BUF) as *mut u8;
        core::ptr::write_bytes(l_buf_ptr, 0, K_INTERFACE_NAME_BUF_LEN);
    }

    match unsafe {
        get_interface_name(
            p_id as u8,
            core::ptr::addr_of_mut!(G_INTERFACE_NAME_BUF) as *mut u8,
        )
    } {
        HalInterfaceResult::OK => {
            let l_buf_ptr = core::ptr::addr_of!(G_INTERFACE_NAME_BUF) as *const u8;
            let mut l_len = 0;
            while l_len < K_INTERFACE_NAME_BUF_LEN {
                let l_byte = unsafe { core::ptr::read(l_buf_ptr.add(l_len)) };
                if l_byte == 0 {
                    break;
                }
                l_len += 1;
            }
            let l_static_bytes: &'static [u8] =
                unsafe { core::slice::from_raw_parts(l_buf_ptr, l_len) };
            let l_static_str = unsafe { core::str::from_utf8_unchecked(l_static_bytes) };
            Ok(l_static_str)
        }
        _ => Err(WrongInterfaceId(p_id)),
    }
}
