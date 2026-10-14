//! Knuth's up-arrow notation, as far as it can be computed.

/// The value of a tower `base ↑↑ height`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tower {
    /// The base.
    pub base: u32,
    /// The height.
    pub height: u32,
    /// The decimal digits, if there are at most the requested maximum.
    pub exact: Option<String>,
    /// The number of decimal digits, if it can be computed exactly.
    pub digits: Option<u128>,
}

/// Computes `base ↑↑ height` = base^base^...^base (`height` copies).
///
/// ```
/// use turing_bb::uparrow::tetration;
/// assert_eq!(tetration(2, 4, 100).exact.as_deref(), Some("65536"));
/// assert_eq!(tetration(2, 5, 100).digits, Some(19_729));
/// ```
pub fn tetration(base: u32, height: u32, max_digits: usize) -> Tower {
    let _ = (base, height, max_digits);
    todo!("A3")
}
