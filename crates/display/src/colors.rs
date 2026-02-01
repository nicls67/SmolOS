use hal_interface::PixelColorARGB;

/// High-level enumeration of supported colors.
#[derive(Copy, Clone, Debug)]
pub enum Colors {
    /// Black (0, 0, 0)
    Black,
    /// White (255, 255, 255)
    White,
    /// Red (255, 0, 0)
    Red,
    /// Green (0, 255, 0)
    Green,
    /// Blue (0, 0, 255)
    Blue,
    /// Yellow (255, 255, 0)
    Yellow,
    /// Cyan (0, 255, 255)
    Cyan,
    /// Magenta (255, 0, 255)
    Magenta,
}

impl Colors {
    /// Converts the high-level color to its ARGB representation.
    ///
    /// # Returns
    /// A `PixelColorARGB` structure representing the color.
    pub fn to_argb(&self) -> PixelColorARGB {
        match self {
            Colors::Black => PixelColorARGB::from_u32(0xFF000000),
            Colors::White => PixelColorARGB::from_u32(0xFFFFFFFF),
            Colors::Red => PixelColorARGB::from_u32(0xFFFF0000),
            Colors::Green => PixelColorARGB::from_u32(0xFF00FF00),
            Colors::Blue => PixelColorARGB::from_u32(0xFF0000FF),
            Colors::Yellow => PixelColorARGB::from_u32(0xFFFFFF00),
            Colors::Cyan => PixelColorARGB::from_u32(0xFF00FFFF),
            Colors::Magenta => PixelColorARGB::from_u32(0xFFFF00FF),
        }
    }
}
