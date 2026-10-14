//! Source positions at the boundary.
//!
//! The workspace crates measure positions in UTF-8 bytes ([`turing_core::Span`]); JavaScript
//! strings and `CodeMirror` measure them in UTF-16 code units. The two differ as soon as a source
//! contains δ, Σ, Γ or ␣: `δ` is two bytes but one code unit, `␣` three bytes but one code unit,
//! and characters outside the Basic Multilingual Plane are four bytes and two code units.
//!
//! # Convention
//!
//! * When the bindings parse text themselves (`parseSource`, `fromStandard`, `decodeShare`),
//!   they know the source, so every byte span of the core is converted to UTF-16 and diagnostics
//!   also get a 1-based line and a 1-based column counted in UTF-16 code units
//!   ([`Positions::Source`]).
//! * When a spec comes from JavaScript (`checkSpec`, `formatSpec`, `decide`, ...), its
//!   transition spans already are UTF-16 offsets into the editor text, and the bindings have no
//!   source to convert with. Such spans travel through the core unchanged (stored in the byte
//!   fields of [`Span`]) and come back out unchanged, without line and column
//!   ([`Positions::Utf16`]). The core only copies spans into diagnostics and never does
//!   arithmetic on them, so the round trip is exact.

use turing_core::Span;

use crate::dto::SpanDto;

/// Converts UTF-8 byte offsets of one source text to UTF-16 offsets and line and column.
///
/// Building the index is linear in the source; a conversion is a binary search over the lines
/// plus a scan of one line.
#[derive(Clone, Debug)]
pub(crate) struct LineIndex<'a> {
    source: &'a str,
    /// Byte offset of the first character of each line.
    starts: Vec<usize>,
    /// UTF-16 offset of the first character of each line.
    starts16: Vec<u32>,
}

impl<'a> LineIndex<'a> {
    /// Indexes the line starts of `source`. Lines end after `\n`; a `\r` before it belongs to
    /// the line it ends.
    pub(crate) fn new(source: &'a str) -> Self {
        let mut starts = vec![0];
        let mut starts16 = vec![0];
        let mut units: u32 = 0;
        for (i, c) in source.char_indices() {
            units = units.saturating_add(utf16_len(c));
            if c == '\n' {
                starts.push(i + 1);
                starts16.push(units);
            }
        }
        Self {
            source,
            starts,
            starts16,
        }
    }

    /// The UTF-16 offset of byte offset `byte`.
    ///
    /// Offsets past the end are clamped to the end, and offsets inside a multi-byte character
    /// are moved back to its first byte, so every input gives a valid position.
    pub(crate) fn utf16(&self, byte: u32) -> u32 {
        let (line, byte) = self.locate(byte);
        let start = self.starts[line];
        let column: u32 = self.source[start..byte].chars().map(utf16_len).sum();
        self.starts16[line].saturating_add(column)
    }

    /// The 1-based line and 1-based UTF-16 column of byte offset `byte`, clamped like
    /// [`LineIndex::utf16`].
    pub(crate) fn line_column(&self, byte: u32) -> (u32, u32) {
        let (line, byte) = self.locate(byte);
        let start = self.starts[line];
        let column: u32 = self.source[start..byte].chars().map(utf16_len).sum();
        (saturate(line + 1), column.saturating_add(1))
    }

    /// The line index containing `byte` and `byte` clamped to a character boundary.
    fn locate(&self, byte: u32) -> (usize, usize) {
        let mut byte = usize::try_from(byte)
            .unwrap_or(usize::MAX)
            .min(self.source.len());
        while !self.source.is_char_boundary(byte) {
            byte -= 1;
        }
        // `starts[0] == 0 <= byte`, so the partition point is at least 1.
        let line = self
            .starts
            .partition_point(|&s| s <= byte)
            .saturating_sub(1);
        (line, byte)
    }
}

/// How the spans of a conversion are interpreted, see the module documentation.
#[derive(Clone, Debug)]
pub(crate) enum Positions<'a> {
    /// Spans are UTF-8 byte offsets into this source.
    Source(LineIndex<'a>),
    /// Spans are UTF-16 offsets already and pass through unchanged.
    Utf16,
}

impl<'a> Positions<'a> {
    /// Positions in `source`.
    pub(crate) fn source(source: &'a str) -> Self {
        Self::Source(LineIndex::new(source))
    }

    /// The UTF-16 range of `span`.
    pub(crate) fn span(&self, span: Span) -> SpanDto {
        match self {
            Self::Source(index) => SpanDto {
                from: index.utf16(span.start),
                to: index.utf16(span.end),
            },
            Self::Utf16 => SpanDto {
                from: span.start,
                to: span.end,
            },
        }
    }

    /// The 1-based line and UTF-16 column of the start of `span`, if the source is known.
    pub(crate) fn line_column(&self, span: Span) -> Option<(u32, u32)> {
        match self {
            Self::Source(index) => Some(index.line_column(span.start)),
            Self::Utf16 => None,
        }
    }
}

/// The span of a DTO in the core type, unchanged (UTF-16 offsets stored in the byte fields).
pub(crate) fn from_dto(span: SpanDto) -> Span {
    Span::new(span.from, span.to)
}

/// The number of UTF-16 code units of `c`: 1 or 2.
fn utf16_len(c: char) -> u32 {
    if u32::from(c) >= 0x1_0000 { 2 } else { 1 }
}

fn saturate(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The UTF-16 offset of every character boundary, computed the way JavaScript does.
    fn js_offsets(source: &str) -> Vec<(u32, u32)> {
        source
            .char_indices()
            .map(|(i, _)| i)
            .chain([source.len()])
            .map(|i| {
                let units = source[..i].encode_utf16().count();
                (u32::try_from(i).unwrap(), u32::try_from(units).unwrap())
            })
            .collect()
    }

    #[test]
    fn ascii_offsets_are_unchanged() {
        let source = "abc\ndef\n";
        let index = LineIndex::new(source);
        for byte in 0..=8 {
            assert_eq!(index.utf16(byte), byte);
        }
        assert_eq!(index.line_column(0), (1, 1));
        assert_eq!(index.line_column(4), (2, 1));
        assert_eq!(index.line_column(6), (2, 3));
        assert_eq!(index.line_column(8), (3, 1));
    }

    #[test]
    fn course_notation_matches_javascript() {
        let source = "Σ = {x}\nδ(q0, ␣) = (q1, x, R)\nδ(q1, x) = (q1, 𝑥, L)\n";
        let index = LineIndex::new(source);
        for (byte, units) in js_offsets(source) {
            assert_eq!(index.utf16(byte), units, "byte {byte}");
        }
    }

    #[test]
    fn line_and_column_count_utf16_units() {
        let source = "δ(q0, ␣) = (q1, x, R)\n  δ(q1, x) = (q2, ␣, L)";
        let index = LineIndex::new(source);
        // `q1` in the first line: after `δ(q0, ␣) = (`, which is 12 characters.
        let q1 = u32::try_from(source.find("q1").unwrap()).unwrap();
        assert_eq!(index.line_column(q1), (1, 13));
        assert_eq!(index.utf16(q1), 12);
        // `q2` in the second line.
        let q2 = u32::try_from(source.find("q2").unwrap()).unwrap();
        let line_start = source.find('\n').unwrap() + 1;
        let expected = u32::try_from(source[line_start..].find("q2").unwrap()).unwrap();
        let col = u32::try_from(
            source[line_start..][..usize::try_from(expected).unwrap()]
                .chars()
                .count(),
        )
        .unwrap();
        assert_eq!(index.line_column(q2), (2, col + 1));
    }

    #[test]
    fn surrogate_pairs_count_twice() {
        let source = "𝑥y";
        let index = LineIndex::new(source);
        assert_eq!(index.utf16(4), 2);
        assert_eq!(index.line_column(4), (1, 3));
    }

    #[test]
    fn crlf_lines() {
        let source = "a\r\nδ\r\nb";
        let index = LineIndex::new(source);
        let b = u32::try_from(source.find('b').unwrap()).unwrap();
        assert_eq!(index.line_column(b), (3, 1));
        assert_eq!(index.utf16(b), 6);
    }

    #[test]
    fn out_of_range_and_inner_offsets_are_clamped() {
        let source = "δ␣";
        let index = LineIndex::new(source);
        // Inside `δ` (bytes 0..2) and inside `␣` (bytes 2..5).
        assert_eq!(index.utf16(1), 0);
        assert_eq!(index.utf16(3), 1);
        assert_eq!(index.utf16(4), 1);
        assert_eq!(index.utf16(5), 2);
        assert_eq!(index.utf16(1_000), 2);
        assert_eq!(index.utf16(u32::MAX), 2);
        assert_eq!(index.line_column(u32::MAX), (1, 3));
        let empty = LineIndex::new("");
        assert_eq!(empty.utf16(7), 0);
        assert_eq!(empty.line_column(7), (1, 1));
    }

    #[test]
    fn positions_pass_utf16_through() {
        let span = Span::new(3, 9);
        assert_eq!(Positions::Utf16.span(span), SpanDto { from: 3, to: 9 });
        assert_eq!(Positions::Utf16.line_column(span), None);
        assert_eq!(from_dto(SpanDto { from: 3, to: 9 }), span);
        let source = Positions::source("δδδδ");
        assert_eq!(source.span(Span::new(2, 6)), SpanDto { from: 1, to: 3 });
        assert_eq!(source.line_column(Span::new(2, 6)), Some((1, 2)));
    }

    #[test]
    fn every_boundary_round_trips() {
        let source = "// Σ, Γ and ␣\nδ(q0, (x, ␣)) = (q1, (x, x), (R, N))\r\n𝑥\n";
        let index = LineIndex::new(source);
        let mut previous = None;
        for (byte, units) in js_offsets(source) {
            assert_eq!(index.utf16(byte), units);
            // Offsets are strictly increasing over character boundaries.
            if let Some(p) = previous {
                assert!(units > p);
            }
            previous = Some(units);
        }
    }
}
