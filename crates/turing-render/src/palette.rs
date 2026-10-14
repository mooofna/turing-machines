//! Colours for space-time diagrams.

use std::fmt;

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
        let digits = text.strip_prefix('#')?.as_bytes();
        let nibble = |i: usize| -> Option<u8> {
            let value = char::from(*digits.get(i)?).to_digit(16)?;
            u8::try_from(value).ok()
        };
        match digits.len() {
            6 => Some(Self(
                (nibble(0)? << 4) | nibble(1)?,
                (nibble(2)? << 4) | nibble(3)?,
                (nibble(4)? << 4) | nibble(5)?,
            )),
            3 => Some(Self(nibble(0)? * 17, nibble(1)? * 17, nibble(2)? * 17)),
            _ => None,
        }
    }

    /// The colour as `#rrggbb` in lower case.
    ///
    /// ```
    /// use turing_render::Rgb;
    /// assert_eq!(Rgb(0x21, 0x5c, 0xaf).to_hex(), "#215caf");
    /// ```
    pub fn to_hex(self) -> String {
        self.to_string()
    }

    /// The colour `t` of the way from `self` to `other`, mixed per channel in sRGB.
    ///
    /// `t` is clamped to 0 to 1; NaN counts as 0.
    ///
    /// ```
    /// use turing_render::Rgb;
    /// let grey = Rgb(255, 255, 255).mix(Rgb(0, 0, 0), 0.5);
    /// assert_eq!(grey, Rgb(128, 128, 128));
    /// ```
    #[must_use]
    pub fn mix(self, other: Self, t: f64) -> Self {
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
        let channel = |a: u8, b: u8| -> u8 {
            let value = f64::from(a) + (f64::from(b) - f64::from(a)) * t;
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a mix of two bytes, rounded and clamped, is a byte"
            )]
            let byte = value.round().clamp(0.0, 255.0) as u8;
            byte
        };
        Self(
            channel(self.0, other.0),
            channel(self.1, other.1),
            channel(self.2, other.2),
        )
    }
}

impl fmt::Display for Rgb {
    /// Writes `#rrggbb`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
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

impl Palette {
    /// The light theme of the site: white paper, near-black ink, ETH blue.
    pub const LIGHT: Self = Self {
        background: Rgb(0xff, 0xff, 0xff),
        ink: Rgb(0x11, 0x11, 0x11),
        accent: Rgb(0x21, 0x5c, 0xaf),
    };

    /// The dark theme of the site: near-black paper, near-white ink, a lighter blue.
    pub const DARK: Self = Self {
        background: Rgb(0x0e, 0x0e, 0x0e),
        ink: Rgb(0xf1, 0xf1, 0xf1),
        accent: Rgb(0x7b, 0xa7, 0xf0),
    };

    /// A palette from three CSS hex colours, as accepted by [`Rgb::from_hex`].
    ///
    /// ```
    /// use turing_render::Palette;
    /// assert_eq!(Palette::from_hex("#fff", "#111111", "#215caf"), Some(Palette::LIGHT));
    /// assert_eq!(Palette::from_hex("white", "#111", "#215caf"), None);
    /// ```
    pub fn from_hex(background: &str, ink: &str, accent: &str) -> Option<Self> {
        Some(Self {
            background: Rgb::from_hex(background)?,
            ink: Rgb::from_hex(ink)?,
            accent: Rgb::from_hex(accent)?,
        })
    }
}

impl Default for Palette {
    /// [`Palette::LIGHT`].
    fn default() -> Self {
        Self::LIGHT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_long_and_short_hex() {
        assert_eq!(Rgb::from_hex("#000000"), Some(Rgb(0, 0, 0)));
        assert_eq!(Rgb::from_hex("#FFFFFF"), Some(Rgb(255, 255, 255)));
        assert_eq!(Rgb::from_hex("#7ba7f0"), Some(Rgb(0x7b, 0xa7, 0xf0)));
        assert_eq!(Rgb::from_hex("#a1c"), Some(Rgb(0xaa, 0x11, 0xcc)));
    }

    #[test]
    fn rejects_malformed_hex() {
        for text in [
            "",
            "#",
            "fff",
            "#ff",
            "#ffff",
            "#fffff",
            "#fffffff",
            "#ggg",
            "#12345g",
            "#\u{e9}\u{e9}",
            " #fff",
            "#fff ",
            "#+12",
        ] {
            assert_eq!(Rgb::from_hex(text), None, "{text:?}");
        }
    }

    #[test]
    fn hex_round_trips() {
        for rgb in [
            Rgb(0, 0, 0),
            Rgb(1, 2, 3),
            Rgb(0x21, 0x5c, 0xaf),
            Rgb(255, 255, 255),
        ] {
            assert_eq!(Rgb::from_hex(&rgb.to_hex()), Some(rgb));
        }
    }

    #[test]
    fn mix_hits_the_ends_and_clamps() {
        let (a, b) = (Rgb(10, 20, 30), Rgb(250, 120, 0));
        assert_eq!(a.mix(b, 0.0), a);
        assert_eq!(a.mix(b, 1.0), b);
        assert_eq!(a.mix(b, -3.0), a);
        assert_eq!(a.mix(b, 7.0), b);
        assert_eq!(a.mix(b, f64::NAN), a);
    }

    #[test]
    fn site_palettes_parse_from_the_css_tokens() {
        assert_eq!(
            Palette::from_hex("#ffffff", "#111111", "#215caf"),
            Some(Palette::LIGHT)
        );
        assert_eq!(
            Palette::from_hex("#0e0e0e", "#f1f1f1", "#7ba7f0"),
            Some(Palette::DARK)
        );
    }
}
