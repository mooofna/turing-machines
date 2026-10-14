//! How many machines there are.
//!
//! Two conventions are in use for counting the machines with n states and k symbols. They
//! differ in how transitions to the halting state are counted.

/// The number of machines with `states` states and `symbols` symbols in Radó's count, where each
/// transition writes one of `symbols` symbols, moves left or right and continues in one of the
/// `states` states or halts: `(2 × symbols × (states + 1)) ^ (states × symbols)`.
///
/// For two symbols this is (4(n + 1))^(2n), the count of Radó's paper *On non-computable
/// functions* (Bell System Technical Journal, 1962). A halting transition still writes a symbol
/// and moves, so it comes in 2k variants.
///
/// Returns `None` on overflow.
///
/// ```
/// use turing_bb::count::machines;
/// assert_eq!(machines(5, 2), Some(63_403_380_965_376));
/// assert_eq!(machines(1, 2), Some(64));
/// ```
pub fn machines(states: u32, symbols: u32) -> Option<u128> {
    let choices = 2u128
        .checked_mul(u128::from(symbols))?
        .checked_mul(u128::from(states) + 1)?;
    choices.checked_pow(states.checked_mul(symbols)?)
}

/// The number of machines with `states` states and `symbols` symbols when there is a single
/// halting transition: each transition is one of the 2 × `symbols` × `states` proper
/// transitions or "halt", so the count is `(2 × symbols × states + 1) ^ (states × symbols)`.
///
/// For two symbols this is (4n + 1)^(2n), the convention of Aaronson, *The Busy Beaver
/// Frontier* (SIGACT News, 2020), and of the bbchallenge collaboration's paper
/// *Determination of the fifth Busy Beaver value* (arXiv:2509.12337, 2025).
///
/// Returns `None` on overflow.
///
/// ```
/// use turing_bb::count::machines_single_halt;
/// assert_eq!(machines_single_halt(5, 2), Some(16_679_880_978_201));
/// assert_eq!(machines_single_halt(1, 2), Some(25));
/// ```
pub fn machines_single_halt(states: u32, symbols: u32) -> Option<u128> {
    let choices = 2u128
        .checked_mul(u128::from(symbols))?
        .checked_mul(u128::from(states))?
        .checked_add(1)?;
    choices.checked_pow(states.checked_mul(symbols)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_counts() {
        assert_eq!(machines(2, 2), Some(12u128.pow(4)));
        assert_eq!(machines(4, 2), Some(20u128.pow(8)));
        assert_eq!(machines(2, 3), Some(18u128.pow(6)));
        assert_eq!(machines_single_halt(2, 2), Some(9u128.pow(4)));
        assert_eq!(machines_single_halt(4, 2), Some(17u128.pow(8)));
        assert_eq!(machines_single_halt(2, 4), Some(17u128.pow(8)));
    }

    #[test]
    fn single_halt_is_never_larger() {
        for n in 1..=5 {
            for k in 2..=4 {
                let single = machines_single_halt(n, k).unwrap();
                assert!(single < machines(n, k).unwrap(), "{n} states, {k} symbols");
            }
        }
    }

    #[test]
    fn overflow_is_reported() {
        assert_eq!(machines(100, 2), None);
        assert_eq!(machines_single_halt(100, 2), None);
        assert_eq!(machines(u32::MAX, 2), None);
        assert_eq!(machines_single_halt(2, u32::MAX), None);
    }
}
