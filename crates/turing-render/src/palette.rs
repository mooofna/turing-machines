//! Colours for space-time diagrams.

/// An sRGB colour.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    /// Parses `#rrggbb` or `#rgb`.
    ///
    /// ```
    /// use turing_render::Rgb;
    /// assert_eq!(Rgb::from_hex("#215caf"), Some(Rgb(0x21, 0x5c, 0xaf)));
    /// assert_eq!(Rgb::from_hex("#fff"), Some(Rgb(255, 255, 255)));
    /// ```
    pub fn from_hex(text: &str) -> Option<Self> {
        let _ = text;
        todo!("A4")
    }
}

/// The three colours of a diagram: the page background, the ink for written cells and the
/// accent for the head.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Palette {
    /// Blank cells.
    pub background: Rgb,
    /// The darkest shade, used for the highest symbol or a full bin.
    pub ink: Rgb,
    /// The head.
    pub accent: Rgb,
}
