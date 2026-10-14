//! One module per subcommand. Each has an `Args` struct for clap and an `exec` function.

pub(crate) mod antihydra;
pub(crate) mod check;
pub(crate) mod convert;
pub(crate) mod decide;
pub(crate) mod enumerate;
pub(crate) mod fmt;
pub(crate) mod render;
pub(crate) mod run;
pub(crate) mod trace;

use turing_core::{Kind, Machine, NonHaltProof, Outcome, State};

use crate::text;

/// Parses a count such as `1000`, `1_000_000`, `1,000,000`, `1e9` or `10^9`.
pub(crate) fn parse_count(s: &str) -> Result<u64, String> {
    let clean: String = s.chars().filter(|&c| c != '_' && c != ',').collect();
    let invalid =
        || format!("`{s}` is not a count; write for example 1000, 1_000_000, 1e9 or 10^9");
    let power = |base: &str, exp: &str| -> Result<u64, String> {
        let base: u64 = base.parse().map_err(|_| invalid())?;
        let exp: u32 = exp.parse().map_err(|_| invalid())?;
        base.checked_pow(exp)
            .ok_or_else(|| format!("`{s}` is too large; the limit is {}", text::num(u64::MAX)))
    };
    if let Some((mantissa, exp)) = clean.split_once(['e', 'E']) {
        let mantissa: u64 = mantissa.parse().map_err(|_| invalid())?;
        let scale = power("10", exp)?;
        mantissa
            .checked_mul(scale)
            .ok_or_else(|| format!("`{s}` is too large; the limit is {}", text::num(u64::MAX)))
    } else if let Some((base, exp)) = clean.split_once('^') {
        power(base, exp)
    } else {
        clean.parse().map_err(|_| invalid())
    }
}

/// `transducer`, `acceptor`, `enumerator` or `beaver`.
pub(crate) fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Acceptor => "acceptor",
        Kind::Transducer => "transducer",
        Kind::Enumerator => "enumerator",
        Kind::Beaver => "beaver",
    }
}

/// The name of state `q`.
pub(crate) fn state_name(machine: &Machine, q: State) -> &str {
    machine
        .states()
        .get(q.index())
        .map_or("?", |info| info.name.as_str())
}

/// `transducer, 1 tape, 10 states, 4 symbols, 19 transitions`.
///
/// Busy beavers are counted without their halting state, as in "the 5-state busy beaver".
pub(crate) fn shape(machine: &Machine) -> String {
    let states = if machine.kind() == Kind::Beaver {
        let halting: Vec<&str> = machine
            .states()
            .iter()
            .enumerate()
            .filter(|&(i, info)| {
                info.is_final && !machine.transitions().any(|t| t.from.index() == i)
            })
            .map(|(_, info)| info.name.as_str())
            .collect();
        let working = machine.num_states() - halting.len();
        let states = text::count(working as u64, "state", "states");
        match halting.as_slice() {
            [z] => format!("{states} and the halting state {z}"),
            _ => states,
        }
    } else {
        text::count(machine.num_states() as u64, "state", "states")
    };
    format!(
        "{}, {}, {states}, {}, {}",
        kind_name(machine.kind()),
        text::count(machine.tapes() as u64, "tape", "tapes"),
        text::count(machine.num_symbols() as u64, "symbol", "symbols"),
        text::count(
            machine.num_transitions() as u64,
            "transition",
            "transitions"
        ),
    )
}

/// Why a machine never halts, in words.
pub(crate) fn proof_text(machine: &Machine, proof: NonHaltProof) -> String {
    match proof {
        NonHaltProof::ChainToInfinity { state, dir } => {
            let way = match dir {
                turing_core::Dir::L => "left",
                turing_core::Dir::R => "right",
                turing_core::Dir::N => "in place",
            };
            format!(
                "in state {} the head moves {way} over blank tape forever",
                state_name(machine, state)
            )
        }
        NonHaltProof::StayLoop { state } => format!(
            "in state {} the machine repeats the same step in place forever",
            state_name(machine, state)
        ),
    }
}

/// `halted after 107 steps in state q9` and the like, without styles.
pub(crate) fn outcome_text(machine: &Machine, outcome: Outcome) -> String {
    match outcome {
        Outcome::Halted { steps, state } => format!(
            "halted after {} in state {}",
            text::count(steps, "step", "steps"),
            state_name(machine, state)
        ),
        Outcome::OutOfFuel { steps } => {
            format!(
                "did not halt within {}",
                text::count(steps, "step", "steps")
            )
        }
        Outcome::NeverHalts { steps, proof } => format!(
            "never halts: {} (proved at step {})",
            proof_text(machine, proof),
            text::num(steps)
        ),
        Outcome::Stopped { steps } => {
            format!("stopped after {}", text::count(steps, "step", "steps"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts() {
        assert_eq!(parse_count("50"), Ok(50));
        assert_eq!(parse_count("1_000_000"), Ok(1_000_000));
        assert_eq!(parse_count("1,000"), Ok(1_000));
        assert_eq!(parse_count("1e9"), Ok(1_000_000_000));
        assert_eq!(parse_count("3E2"), Ok(300));
        assert_eq!(parse_count("10^9"), Ok(1_000_000_000));
        assert_eq!(
            parse_count("2^64").map_err(|e| e.contains("too large")),
            Err(true)
        );
        assert!(parse_count("ten").is_err());
        assert!(parse_count("-1").is_err());
    }
}
