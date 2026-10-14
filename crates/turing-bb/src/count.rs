//! How many machines there are.

/// The number of machines with `states` states and `symbols` symbols in Radó's count, where each
/// transition writes one of `symbols` symbols, moves left or right and continues in one of the
/// `states` states or halts: `(2 × symbols × (states + 1)) ^ (states × symbols)`.
///
/// Returns `None` on overflow.
///
/// ```
/// use turing_bb::count::machines;
/// assert_eq!(machines(5, 2), Some(63_403_380_965_376));
/// ```
pub fn machines(states: u32, symbols: u32) -> Option<u128> {
    let choices = 2u128
        .checked_mul(u128::from(symbols))?
        .checked_mul(u128::from(states) + 1)?;
    choices.checked_pow(states.checked_mul(symbols)?)
}
