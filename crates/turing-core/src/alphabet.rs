//! The tape alphabet of a machine and its input sub-alphabet.

use alloc::vec::Vec;

use crate::builder::BuildError;
use crate::symbol::{MAX_SYMBOLS, Symbol};

/// The tape alphabet Γ with its blank and the input alphabet Σ ⊆ Γ \ {␣}.
///
/// Glyph 0 is the blank. The remaining glyphs keep the order in which they were given.
///
/// ```
/// use turing_core::Alphabet;
/// let a = Alphabet::new('␣', &['x'], &['x', 'X']).unwrap();
/// assert_eq!(a.len(), 3);
/// assert_eq!(a.glyph(a.symbol('X').unwrap()), 'X');
/// assert!(a.is_input(a.symbol('x').unwrap()));
/// assert!(!a.is_input(a.symbol('X').unwrap()));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alphabet {
    glyphs: Vec<char>,
    input: Vec<bool>,
}

impl Alphabet {
    /// Builds Γ = {blank} ∪ `tape` ∪ `input` and marks `input` as Σ.
    ///
    /// # Errors
    ///
    /// Fails if the blank is part of the input alphabet or if Γ has more than
    /// [`MAX_SYMBOLS`] glyphs.
    pub fn new(blank: char, input: &[char], tape: &[char]) -> Result<Self, BuildError> {
        if input.contains(&blank) {
            return Err(BuildError::BlankInInputAlphabet { blank });
        }
        let mut glyphs = Vec::with_capacity(tape.len() + input.len() + 1);
        glyphs.push(blank);
        for &c in tape.iter().chain(input) {
            if !glyphs.contains(&c) {
                glyphs.push(c);
            }
        }
        if glyphs.len() > MAX_SYMBOLS {
            return Err(BuildError::TooManySymbols { max: MAX_SYMBOLS });
        }
        let input = glyphs.iter().map(|c| input.contains(c)).collect();
        Ok(Self { glyphs, input })
    }

    /// The alphabet `{0, 1, ..., k - 1}` with blank `0` and input `{1, ..., k - 1}`, as used by
    /// busy beaver machines.
    ///
    /// # Panics
    ///
    /// Panics if `k` is 0 or larger than 10.
    ///
    /// ```
    /// use turing_core::Alphabet;
    /// let a = Alphabet::digits(2);
    /// assert_eq!(a.blank(), '0');
    /// assert_eq!(a.len(), 2);
    /// ```
    pub fn digits(k: usize) -> Self {
        assert!(
            (1..=10).contains(&k),
            "digit alphabets have 1 to 10 symbols"
        );
        let glyphs: Vec<char> = (0..k)
            .map(|i| char::from_digit(u32::try_from(i).unwrap_or(0), 10).unwrap_or('0'))
            .collect();
        let input = (0..k).map(|i| i > 0).collect();
        Self { glyphs, input }
    }

    /// The number of symbols in Γ, blank included.
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    /// Always `false`: Γ contains at least the blank.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// The blank glyph.
    pub fn blank(&self) -> char {
        self.glyphs[0]
    }

    /// The symbol written as `glyph`, if it belongs to Γ.
    pub fn symbol(&self, glyph: char) -> Option<Symbol> {
        self.glyphs
            .iter()
            .position(|&c| c == glyph)
            .and_then(|i| u8::try_from(i).ok())
            .map(Symbol::new)
    }

    /// The glyph of `symbol`. Unknown symbols are shown as `?`.
    pub fn glyph(&self, symbol: Symbol) -> char {
        self.glyphs.get(symbol.index()).copied().unwrap_or('?')
    }

    /// Whether `symbol` belongs to the input alphabet Σ.
    pub fn is_input(&self, symbol: Symbol) -> bool {
        self.input.get(symbol.index()).copied().unwrap_or(false)
    }

    /// All glyphs in index order, blank first.
    pub fn glyphs(&self) -> &[char] {
        &self.glyphs
    }

    /// The input alphabet Σ in index order.
    pub fn input_glyphs(&self) -> impl Iterator<Item = char> + '_ {
        self.glyphs
            .iter()
            .zip(&self.input)
            .filter_map(|(&c, &is_input)| is_input.then_some(c))
    }

    /// All symbols with their glyphs, blank first.
    pub fn iter(&self) -> impl Iterator<Item = (Symbol, char)> + '_ {
        self.glyphs
            .iter()
            .enumerate()
            .filter_map(|(i, &c)| u8::try_from(i).ok().map(|i| (Symbol::new(i), c)))
    }
}
