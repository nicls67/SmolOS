use hal_interface::PixelColorARGB;

#[derive(Copy, Clone)]
pub enum Colors {
    Black,
    White,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
}

impl Colors {
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
