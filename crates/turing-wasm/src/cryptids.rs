//! Antihydra: checking its rules on the machine and iterating the rule itself.

use turing_bb::cryptids::antihydra::{MachineCheck, Row, RuleWalk};

use crate::convert::{number, signed};
use crate::dto::{AntihydraRowDto, AntihydraRowsDto, WalkSampleDto, WalkSummaryDto};

/// The largest number of samples a walk summary returns.
pub(crate) const MAX_SAMPLES: u32 = 10_000;

/// Runs at most `max_macro_steps` macro steps of the machine check.
pub(crate) fn check(job: &mut MachineCheck, max_macro_steps: u64) -> AntihydraRowsDto {
    let rows = if job.is_done() {
        Vec::new()
    } else {
        job.advance(max_macro_steps).iter().map(row).collect()
    };
    AntihydraRowsDto {
        rows,
        done: job.is_done(),
    }
}

fn row(r: &Row) -> AntihydraRowDto {
    AntihydraRowDto {
        iteration: r.iteration,
        a: number(r.a),
        b: number(r.b),
        steps: number(r.steps),
        delta: number(r.delta),
        predicted: number(r.predicted),
        agrees: r.agrees,
    }
}

/// Runs `iterations` more iterations of the rule and summarises the walk with at most
/// `samples` samples (capped at [`MAX_SAMPLES`]).
pub(crate) fn walk(job: &mut RuleWalk, iterations: u64, samples: u32) -> WalkSummaryDto {
    if iterations > 0 {
        job.advance(iterations);
    }
    let s = job.summary(usize::try_from(samples.min(MAX_SAMPLES)).unwrap_or(0));
    WalkSummaryDto {
        iterations: number(s.iterations),
        halted: s.halted,
        evens: number(s.evens),
        odds: number(s.odds),
        balance: signed(s.balance),
        min_balance: signed(s.min_balance),
        bits: number(s.bits),
        samples: s
            .samples
            .iter()
            .map(|p| WalkSampleDto {
                iteration: number(p.iteration),
                balance: signed(p.balance),
                bits: number(p.bits),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_check_agrees_with_the_rules() {
        let mut job = MachineCheck::new();
        let mut rows = Vec::new();
        while rows.len() < 20 {
            let batch = check(&mut job, 10_000);
            rows.extend(batch.rows);
            assert!(!batch.done || rows.len() >= 20);
        }
        assert_eq!(rows[0].a, 0.0);
        assert_eq!(rows[0].b, 4.0);
        assert!(rows.iter().all(|r| r.agrees), "{rows:?}");
        for pair in rows.windows(2) {
            assert_eq!(pair[1].iteration, pair[0].iteration + 1);
            assert_eq!(pair[1].delta, pair[1].steps - pair[0].steps);
        }
    }

    #[test]
    fn rule_walk_summaries() {
        let mut job = RuleWalk::new();
        let s = walk(&mut job, 1_000, 10);
        assert_eq!(s.iterations, 1_000.0);
        assert!(!s.halted);
        assert_eq!(s.evens + s.odds, 1_000.0);
        assert!(s.samples.len() <= 10);
        let s = walk(&mut job, 0, u32::MAX);
        assert_eq!(s.iterations, 1_000.0);
        assert!(s.samples.len() <= MAX_SAMPLES as usize);
    }
}
