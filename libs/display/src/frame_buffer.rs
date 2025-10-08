use hal_interface::Hal;

const FRAME_BUFFER_1_ADDRESS: u32 = 0xC0000000;
const FRAME_BUFFER_2_ADDRESS: u32 = 0xC0200000;

pub enum FrameBufferSelector {
    FrameBuffer1,
    FrameBuffer2,
}

pub struct FrameBuffer {
    selected: FrameBufferSelector,
}

impl FrameBuffer {
    /// Constructs a new instance of the struct with default values.
    ///
    /// # Returns
    /// A new instance of the struct where:
    /// - `selected` is set to `FrameBufferSelector::FrameBuffer2`.
    ///
    pub fn new() -> Self {
        Self {
            selected: FrameBufferSelector::FrameBuffer2,
        }
    }

    /// Returns the memory address of the currently active frame buffer.
    ///
    /// This function checks the currently selected frame buffer and returns the corresponding
    /// memory address. The selection is based on the value of the `self.selected` field, which
    /// determines the active frame buffer.
    ///
    /// # Returns
    /// * `FRAME_BUFFER_1_ADDRESS` if `self.selected` is `FrameBufferSelector::FrameBuffer1`.
    /// * `FRAME_BUFFER_2_ADDRESS` if `self.selected` is `FrameBufferSelector::FrameBuffer2`.
    ///
    /// # Assumptions
    /// This function assumes that the `self.selected` field is properly initialized
    /// and holds a valid `FrameBufferSelector` value.
    ///
    /// # Errors
    /// This function does not return any errors and assumes the selected frame buffer
    /// always maps to a valid address.
    ///
    /// # Requirements
    /// Ensure that the constants `FRAME_BUFFER_1_ADDRESS` and `FRAME_BUFFER_2_ADDRESS`
    /// are defined in the same scope or accessible to this function.
    pub fn address_active(&self) -> u32 {
        match self.selected {
            FrameBufferSelector::FrameBuffer1 => FRAME_BUFFER_1_ADDRESS,
            FrameBufferSelector::FrameBuffer2 => FRAME_BUFFER_2_ADDRESS,
        }
    }

    /// Returns the memory address of the currently displayed frame buffer.
    ///
    /// This method determines which frame buffer is currently being displayed
    /// based on the value of the `selected` field in the instance. The displayed
    /// frame buffer is the one not currently selected, following an assumed
    /// double-buffering mechanism where one buffer is used for rendering while
    /// the other is displayed.
    ///
    /// # Returns
    /// * `FRAME_BUFFER_2_ADDRESS` - If the selected frame buffer is `FrameBuffer1`.
    /// * `FRAME_BUFFER_1_ADDRESS` - If the selected frame buffer is `FrameBuffer2`.
    ///
    /// # Note
    /// Ensure that the `selected` field is set correctly to represent the current
    /// rendering buffer before calling this method.
    ///
    /// # Dependencies
    /// This function relies on the `FrameBufferSelector` enum and the constants
    /// `FRAME_BUFFER_1_ADDRESS` and `FRAME_BUFFER_2_ADDRESS` being defined.
    pub fn address_displayed(&self) -> u32 {
        match self.selected {
            FrameBufferSelector::FrameBuffer1 => FRAME_BUFFER_2_ADDRESS,
            FrameBufferSelector::FrameBuffer2 => FRAME_BUFFER_1_ADDRESS,
        }
    }

    /// Switches the currently selected frame buffer and returns the address of the displayed frame.
    ///
    /// # Description
    /// This function toggles between two frame buffers, `FrameBuffer1` and `FrameBuffer2`.
    /// When called, it checks the current frame buffer stored in `self.selected` and switches it to
    /// the other buffer. After switching, it returns the address of the frame buffer that is now active
    /// by calling the `address_displayed` method.
    ///
    /// # Returns
    /// A `u32` value representing the address of the currently displayed frame buffer after the switch.
    ///
    pub fn switch(&mut self) -> u32 {
        match self.selected {
            FrameBufferSelector::FrameBuffer1 => self.selected = FrameBufferSelector::FrameBuffer2,
            FrameBufferSelector::FrameBuffer2 => self.selected = FrameBufferSelector::FrameBuffer1,
        }
        self.address_displayed()
    }
}
