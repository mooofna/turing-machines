//! Prints the funnel of an enumeration: `cargo run --release -p turing-bb --example funnel -- 4 2`.

use std::time::Instant;

use turing_bb::Enumerator;

fn main() {
    let mut args = std::env::args().skip(1).map(|a| a.parse::<u8>());
    let states = args.next().and_then(Result::ok).unwrap_or(3);
    let symbols = args.next().and_then(Result::ok).unwrap_or(2);
    let start = Instant::now();
    let mut e = Enumerator::new(states, symbols).expect("supported size");
    let result = e.run_to_completion().clone();
    let elapsed = start.elapsed();
    #[expect(clippy::print_stdout, reason = "an example that prints its result")]
    {
        println!(
            "BB({states},{symbols}): {} nodes in {elapsed:.2?}",
            result.nodes
        );
        println!(
            "halting {}, non-halting {}, holdouts {}",
            result.halting,
            result.non_halting(),
            result.holdouts
        );
        for (name, n) in &result.by_stage {
            println!("  {name:40} {n:>8}");
        }
        if let Some((s, tm)) = result.max_steps {
            println!("S = {s} ({tm})");
        }
        if let Some((s, tm)) = result.max_ones {
            println!("Σ = {s} ({tm})");
        }
        for tm in e.holdouts().iter().take(20) {
            println!("holdout {tm}");
        }
    }
}
