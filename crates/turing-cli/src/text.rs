//! Small text helpers: numbers with thousands separators, durations and plurals.

use std::time::Duration;

/// Writes `n` with commas between groups of three digits, as in `47,176,870`.
pub(crate) fn num(n: impl Into<u128>) -> String {
    let digits = n.into().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Like [`num`] for signed numbers.
pub(crate) fn signed(n: i64) -> String {
    if n < 0 {
        format!("-{}", num(n.unsigned_abs()))
    } else {
        num(n.unsigned_abs())
    }
}

/// `1 state`, `2 states`, with the count formatted by [`num`].
pub(crate) fn count(n: impl Into<u128>, singular: &str, plural: &str) -> String {
    let n = n.into();
    format!("{} {}", num(n), if n == 1 { singular } else { plural })
}

/// A wall-clock duration in the most readable unit.
pub(crate) fn duration(d: Duration) -> String {
    let secs = d.as_secs_f64();
    if secs >= 60.0 {
        let whole = d.as_secs();
        format!("{} min {} s", whole / 60, whole % 60)
    } else if secs >= 1.0 {
        format!("{secs:.2} s")
    } else if secs >= 1e-3 {
        format!("{:.1} ms", secs * 1e3)
    } else {
        format!("{:.0} µs", secs * 1e6)
    }
}

/// Whether wall-clock times should be printed.
///
/// Setting the environment variable `TM_NO_TIMING` to anything but `0` or the empty string turns
/// them off, which makes the output reproducible for tests and documentation.
pub(crate) fn timing_enabled() -> bool {
    std::env::var_os("TM_NO_TIMING").is_none_or(|v| v.is_empty() || v == "0")
}

/// The empty word is written ε, as in the course.
pub(crate) fn word(w: &str) -> &str {
    if w.is_empty() { "ε" } else { w }
}

/// Glyphs as a set in course notation, for example `{x, #}`.
pub(crate) fn glyph_set(glyphs: impl IntoIterator<Item = char>) -> String {
    let inner: Vec<String> = glyphs.into_iter().map(String::from).collect();
    format!("{{{}}}", inner.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_separators() {
        assert_eq!(num(0u8), "0");
        assert_eq!(num(999u32), "999");
        assert_eq!(num(1_000u32), "1,000");
        assert_eq!(num(47_176_870u64), "47,176,870");
        assert_eq!(num(181_385_789u64), "181,385,789");
        assert_eq!(signed(-1_234), "-1,234");
        assert_eq!(signed(12), "12");
    }

    #[test]
    fn plurals() {
        assert_eq!(count(1u8, "tape", "tapes"), "1 tape");
        assert_eq!(count(2_000u32, "state", "states"), "2,000 states");
    }

    #[test]
    fn durations() {
        assert_eq!(duration(Duration::from_micros(250)), "250 µs");
        assert_eq!(duration(Duration::from_micros(12_340)), "12.3 ms");
        assert_eq!(duration(Duration::from_millis(2_500)), "2.50 s");
        assert_eq!(duration(Duration::from_secs(125)), "2 min 5 s");
    }

    #[test]
    fn sets_and_words() {
        assert_eq!(glyph_set(['x', '#']), "{x, #}");
        assert_eq!(word(""), "ε");
        assert_eq!(word("xx"), "xx");
    }
}
