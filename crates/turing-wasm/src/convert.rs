//! Conversions between the DTOs of the boundary and the types of the workspace crates.
//!
//! Glyphs cross the boundary as strings and must be exactly one Unicode scalar value (one Rust
//! `char`, which may be two UTF-16 code units). Everything else maps one to one. Numbers cross
//! as JavaScript `number`s, which are exact up to 2^53; see [`count`] and [`number`] for the
//! rules at that limit.

use std::fmt::Write as _;

use turing_core::{
    BitOrder, Declared, Decoder, Diagnostic, Dir, Kind, MachineSpec, Severity, SpecComments,
    StateSpec, TransitionSpec,
};

use crate::Error;
use crate::dto::{
    BitOrderDto, CommentsDto, DeclaredDto, DecoderDto, DiagnosticDto, DirDto, KindDto,
    MachineSpecDto, RelatedDto, SeverityDto, StateDto, TransitionDto,
};
use crate::spans::{self, Positions};

/// The largest integer a JavaScript `number` holds exactly, 2^53 - 1.
pub(crate) const MAX_SAFE: u64 = (1 << 53) - 1;

/// [`MAX_SAFE`] as a `number`.
const MAX_SAFE_NUMBER: f64 = 9_007_199_254_740_991.0;

/// The diagnostic code for a spec from JavaScript with a malformed glyph.
pub(crate) const CODE_GLYPH: &str = "E102";

/// A spec from JavaScript that cannot be converted, with the location of the problem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Invalid {
    /// What is wrong, as one sentence.
    pub(crate) message: String,
    /// The UTF-16 span of the transition concerned, if it has one.
    pub(crate) span: Option<crate::dto::SpanDto>,
}

impl Invalid {
    /// A diagnostic for `checkSpec`.
    pub(crate) fn to_diagnostic(&self) -> DiagnosticDto {
        DiagnosticDto {
            severity: SeverityDto::Error,
            code: CODE_GLYPH.into(),
            message: self.message.clone(),
            help: Some("Write each symbol as exactly one character.".into()),
            from: self.span.map(|s| s.from),
            to: self.span.map(|s| s.to),
            line: None,
            column: None,
            related: Vec::new(),
        }
    }
}

/// The first problem as an [`Error`] for the functions that throw.
pub(crate) fn first_error(problems: &[Invalid]) -> Error {
    problems.first().map_or_else(
        || Error::new("the machine is invalid"),
        |p| Error::new(p.message.clone()),
    )
}

/// Parses a glyph: a string of exactly one character.
///
/// # Errors
///
/// Describes `what` and the offending text if it is empty or longer.
pub(crate) fn glyph(text: &str, what: &str) -> Result<char, String> {
    single(text).ok_or_else(|| {
        if text.is_empty() {
            format!("{what} must be one character, but it is empty")
        } else {
            format!("{what} must be one character, not `{}`", shorten(text, 16))
        }
    })
}

/// The character of a one-character string.
fn single(text: &str) -> Option<char> {
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Some(c),
        _ => None,
    }
}

/// At most `max` characters of `text`, with `…` if it was cut.
pub(crate) fn shorten(text: &str, max: usize) -> String {
    let mut chars = text.chars();
    let mut out: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() {
        out.push('…');
    }
    out
}

/// A glyph as a one-character string.
pub(crate) fn glyph_text(c: char) -> String {
    c.to_string()
}

/// The move of a DTO.
pub(crate) fn dir(d: DirDto) -> Dir {
    match d {
        DirDto::L => Dir::L,
        DirDto::R => Dir::R,
        DirDto::N => Dir::N,
    }
}

/// The DTO of a move.
pub(crate) fn dir_dto(d: Dir) -> DirDto {
    match d {
        Dir::L => DirDto::L,
        Dir::R => DirDto::R,
        Dir::N => DirDto::N,
    }
}

/// The kind of a DTO.
pub(crate) fn kind(k: KindDto) -> Kind {
    match k {
        KindDto::Acceptor => Kind::Acceptor,
        KindDto::Transducer => Kind::Transducer,
        KindDto::Enumerator => Kind::Enumerator,
        KindDto::Beaver => Kind::Beaver,
    }
}

/// The DTO of a kind.
pub(crate) fn kind_dto(k: Kind) -> KindDto {
    match k {
        Kind::Acceptor => KindDto::Acceptor,
        Kind::Transducer => KindDto::Transducer,
        Kind::Enumerator => KindDto::Enumerator,
        Kind::Beaver => KindDto::Beaver,
    }
}

/// The decoder of a DTO.
///
/// # Errors
///
/// Fails if the tally symbol is not one character.
pub(crate) fn decoder(d: &DecoderDto) -> Result<Decoder, String> {
    Ok(match d {
        DecoderDto::Word => Decoder::Word,
        DecoderDto::Unary { symbol } => Decoder::Unary {
            symbol: glyph(symbol, "the tally symbol of the output")?,
        },
        DecoderDto::Binary { order } => Decoder::Binary {
            order: match order {
                BitOrderDto::Lsb => BitOrder::Lsb,
                BitOrderDto::Msb => BitOrder::Msb,
            },
        },
    })
}

/// The DTO of a decoder.
pub(crate) fn decoder_dto(d: Decoder) -> DecoderDto {
    match d {
        Decoder::Word => DecoderDto::Word,
        Decoder::Unary { symbol } => DecoderDto::Unary {
            symbol: glyph_text(symbol),
        },
        Decoder::Binary { order } => DecoderDto::Binary {
            order: match order {
                BitOrder::Lsb => BitOrderDto::Lsb,
                BitOrder::Msb => BitOrderDto::Msb,
            },
        },
    }
}

/// Converts a spec from JavaScript. Transition spans pass through unchanged (UTF-16, see
/// [`spans`]).
///
/// # Errors
///
/// Returns every malformed glyph, in field order.
pub(crate) fn spec(dto: &MachineSpecDto) -> Result<MachineSpec, Vec<Invalid>> {
    let mut problems = Vec::new();
    // The description is only built for malformed glyphs.
    let mut take = |text: &str, what: &dyn Fn() -> String, span| {
        single(text).unwrap_or_else(|| {
            let message = glyph(text, &what()).err().unwrap_or_default();
            problems.push(Invalid { message, span });
            '?'
        })
    };

    let blank = take(&dto.blank, &|| "the blank".into(), None);
    let input_alphabet = dto
        .input_alphabet
        .iter()
        .map(|g| take(g, &|| "a symbol of the input alphabet".into(), None))
        .collect();
    let tape_alphabet = dto
        .tape_alphabet
        .iter()
        .map(|g| take(g, &|| "a symbol of the tape alphabet".into(), None))
        .collect();
    let transitions = dto
        .transitions
        .iter()
        .enumerate()
        .map(|(index, t)| {
            let n = index + 1;
            let span = t.span;
            let read = t
                .read
                .iter()
                .enumerate()
                .map(|(k, g)| take(g, &|| entry("read", k, t.read.len(), n), span))
                .collect();
            let write = t
                .write
                .iter()
                .enumerate()
                .map(|(k, g)| take(g, &|| entry("written", k, t.write.len(), n), span))
                .collect();
            TransitionSpec {
                from: t.from.clone(),
                read,
                to: t.to.clone(),
                write,
                dir: t.dir.iter().copied().map(dir).collect(),
                comment: t.comment.clone(),
                leading: t.leading.clone(),
                span: span.map(spans::from_dto),
            }
        })
        .collect();
    let output = match decoder(&dto.output) {
        Ok(d) => d,
        Err(message) => {
            problems.push(Invalid {
                message,
                span: None,
            });
            Decoder::Word
        }
    };
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(MachineSpec {
        name: dto.name.clone(),
        kind: kind(dto.kind),
        tapes: dto.tapes,
        blank,
        input_alphabet,
        tape_alphabet,
        states: dto
            .states
            .iter()
            .map(|s| StateSpec {
                name: s.name.clone(),
                is_final: s.is_final,
            })
            .collect(),
        start: dto.start.clone(),
        transitions,
        input: dto.input.clone(),
        output,
        declared: Declared {
            states: dto.declared.states,
            input_alphabet: dto.declared.input_alphabet,
            tape_alphabet: dto.declared.tape_alphabet,
            start: dto.declared.start,
            blank: dto.declared.blank,
        },
        comments: SpecComments {
            header: dto.comments.header.clone(),
            trailing: dto.comments.trailing.clone(),
        },
    })
}

/// "the read symbol of transition 3" or "the second written symbol of transition 3".
fn entry(role: &str, k: usize, len: usize, transition: usize) -> String {
    let mut out = String::from("the ");
    if len > 1 {
        let _ = write!(out, "{} ", ordinal(k + 1));
    }
    let _ = write!(out, "{role} symbol of transition {transition}");
    out
}

fn ordinal(n: usize) -> String {
    match n {
        1 => "first".into(),
        2 => "second".into(),
        3 => "third".into(),
        _ => format!("{n}th"),
    }
}

/// The DTO of a spec, with spans mapped by `positions`.
pub(crate) fn spec_dto(spec: &MachineSpec, positions: &Positions<'_>) -> MachineSpecDto {
    MachineSpecDto {
        name: spec.name.clone(),
        kind: kind_dto(spec.kind),
        tapes: spec.tapes,
        blank: glyph_text(spec.blank),
        input_alphabet: spec.input_alphabet.iter().map(|&c| glyph_text(c)).collect(),
        tape_alphabet: spec.tape_alphabet.iter().map(|&c| glyph_text(c)).collect(),
        states: spec
            .states
            .iter()
            .map(|s| StateDto {
                name: s.name.clone(),
                is_final: s.is_final,
            })
            .collect(),
        start: spec.start.clone(),
        transitions: spec
            .transitions
            .iter()
            .map(|t| TransitionDto {
                from: t.from.clone(),
                read: t.read.iter().map(|&c| glyph_text(c)).collect(),
                to: t.to.clone(),
                write: t.write.iter().map(|&c| glyph_text(c)).collect(),
                dir: t.dir.iter().copied().map(dir_dto).collect(),
                comment: t.comment.clone(),
                leading: t.leading.clone(),
                span: t.span.map(|s| positions.span(s)),
            })
            .collect(),
        input: spec.input.clone(),
        output: decoder_dto(spec.output),
        declared: DeclaredDto {
            states: spec.declared.states,
            input_alphabet: spec.declared.input_alphabet,
            tape_alphabet: spec.declared.tape_alphabet,
            start: spec.declared.start,
            blank: spec.declared.blank,
        },
        comments: CommentsDto {
            header: spec.comments.header.clone(),
            trailing: spec.comments.trailing.clone(),
        },
    }
}

/// The DTO of a diagnostic, with positions mapped by `positions`.
pub(crate) fn diagnostic_dto(d: &Diagnostic, positions: &Positions<'_>) -> DiagnosticDto {
    let range = d.span.map(|s| positions.span(s));
    let location = d.span.and_then(|s| positions.line_column(s));
    DiagnosticDto {
        severity: match d.severity {
            Severity::Error => SeverityDto::Error,
            Severity::Warning => SeverityDto::Warning,
        },
        code: d.code.into(),
        message: d.message.clone(),
        help: d.help.clone(),
        from: range.map(|r| r.from),
        to: range.map(|r| r.to),
        line: location.map(|(line, _)| line),
        column: location.map(|(_, column)| column),
        related: d
            .related
            .iter()
            .map(|(span, note)| {
                let r = positions.span(*span);
                RelatedDto {
                    from: r.from,
                    to: r.to,
                    note: note.clone(),
                }
            })
            .collect(),
    }
}

/// The message of the first error among `diagnostics`, for the functions that throw.
pub(crate) fn first_diagnostic(diagnostics: &[Diagnostic]) -> Error {
    diagnostics
        .iter()
        .find(|d| d.is_error())
        .or_else(|| diagnostics.first())
        .map_or_else(
            || Error::new("the machine has errors"),
            |d| Error::new(d.message.clone()),
        )
}

/// A count from JavaScript: fractions are cut off, `NaN` and negative values count as 0, and
/// values beyond 2^53 - 1 (including infinity) count as 2^53 - 1.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value is finite and in 0..2^53 at the cast"
)]
pub(crate) fn count(value: f64) -> u64 {
    if value.is_nan() || value <= 0.0 {
        0
    } else if value >= MAX_SAFE_NUMBER {
        MAX_SAFE
    } else {
        value as u64
    }
}

/// A count for JavaScript. Values beyond 2^53 - 1 are reported as 2^53 - 1; callers that can
/// reach that limit stop there or document it.
#[expect(clippy::cast_precision_loss, reason = "exact below 2^53")]
pub(crate) fn number(value: u64) -> f64 {
    value.min(MAX_SAFE) as f64
}

/// A signed position or balance for JavaScript, saturated to ±(2^53 - 1) like [`number`].
#[expect(clippy::cast_precision_loss, reason = "exact below 2^53")]
pub(crate) fn signed(value: i64) -> f64 {
    let limit = MAX_SAFE.cast_signed();
    value.clamp(-limit, limit) as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{SpanDto, TransitionDto};

    fn sample() -> MachineSpecDto {
        MachineSpecDto {
            name: "Binary increment".into(),
            kind: KindDto::Transducer,
            tapes: 1,
            blank: "␣".into(),
            input_alphabet: vec!["0".into(), "1".into()],
            tape_alphabet: vec!["␣".into(), "0".into(), "1".into()],
            states: vec![
                StateDto {
                    name: "q0".into(),
                    is_final: false,
                },
                StateDto {
                    name: "q1".into(),
                    is_final: true,
                },
            ],
            start: "q0".into(),
            transitions: vec![
                TransitionDto {
                    from: "q0".into(),
                    read: vec!["1".into()],
                    to: "q0".into(),
                    write: vec!["0".into()],
                    dir: vec![DirDto::R],
                    comment: Some("carry".into()),
                    leading: vec!["least significant bit first".into()],
                    span: Some(SpanDto { from: 40, to: 61 }),
                },
                TransitionDto {
                    from: "q0".into(),
                    read: vec!["␣".into()],
                    to: "q1".into(),
                    write: vec!["1".into()],
                    dir: vec![DirDto::N],
                    comment: None,
                    leading: Vec::new(),
                    span: None,
                },
            ],
            input: "1101".into(),
            output: DecoderDto::Binary {
                order: BitOrderDto::Lsb,
            },
            declared: DeclaredDto {
                states: true,
                input_alphabet: true,
                tape_alphabet: false,
                start: true,
                blank: false,
            },
            comments: CommentsDto {
                header: vec!["header".into()],
                trailing: vec!["trailing".into()],
            },
        }
    }

    #[test]
    fn glyphs_are_single_characters() {
        assert_eq!(glyph("␣", "the blank"), Ok('␣'));
        assert_eq!(glyph("𝑥", "the blank"), Ok('𝑥'));
        assert_eq!(
            glyph("", "the blank"),
            Err("the blank must be one character, but it is empty".into())
        );
        assert_eq!(
            glyph("ab", "the blank"),
            Err("the blank must be one character, not `ab`".into())
        );
        let long = "x".repeat(100_000);
        let message = glyph(&long, "the blank").unwrap_err();
        assert_eq!(
            message,
            "the blank must be one character, not `xxxxxxxxxxxxxxxx…`"
        );
    }

    #[test]
    fn spec_round_trips() {
        let dto = sample();
        let spec = spec(&dto).unwrap();
        assert_eq!(spec.blank, '␣');
        assert_eq!(spec.transitions[0].read, vec!['1']);
        assert_eq!(spec.transitions[1].dir, vec![Dir::N]);
        assert_eq!(
            spec.transitions[0].span,
            Some(turing_core::Span::new(40, 61))
        );
        assert_eq!(spec_dto(&spec, &Positions::Utf16), dto);
    }

    #[test]
    fn source_positions_convert_spans() {
        let mut spec = spec(&sample()).unwrap();
        let source = "δδδδδ";
        spec.transitions[0].span = Some(turing_core::Span::new(2, 6));
        let dto = spec_dto(&spec, &Positions::source(source));
        assert_eq!(dto.transitions[0].span, Some(SpanDto { from: 1, to: 3 }));
    }

    #[test]
    fn every_malformed_glyph_is_reported() {
        let mut dto = sample();
        dto.blank = String::new();
        dto.transitions[0].read = vec!["ab".into()];
        dto.transitions[1].write = vec![String::new()];
        dto.output = DecoderDto::Unary {
            symbol: "xx".into(),
        };
        let problems = spec(&dto).unwrap_err();
        let messages: Vec<&str> = problems.iter().map(|p| p.message.as_str()).collect();
        assert_eq!(
            messages,
            [
                "the blank must be one character, but it is empty",
                "the read symbol of transition 1 must be one character, not `ab`",
                "the written symbol of transition 2 must be one character, but it is empty",
                "the tally symbol of the output must be one character, not `xx`",
            ]
        );
        assert_eq!(problems[1].span, Some(SpanDto { from: 40, to: 61 }));
        let diagnostic = problems[1].to_diagnostic();
        assert_eq!(diagnostic.code, CODE_GLYPH);
        assert_eq!(diagnostic.from, Some(40));
        assert_eq!(first_error(&problems).to_string(), messages[0]);
    }

    #[test]
    fn multi_tape_entries_are_numbered() {
        assert_eq!(
            entry("read", 1, 2, 4),
            "the second read symbol of transition 4"
        );
        assert_eq!(entry("read", 0, 1, 4), "the read symbol of transition 4");
        assert_eq!(
            entry("written", 4, 5, 1),
            "the 5th written symbol of transition 1"
        );
    }

    #[test]
    fn enums_round_trip() {
        for d in [DirDto::L, DirDto::R, DirDto::N] {
            assert_eq!(dir_dto(dir(d)), d);
        }
        for k in [
            KindDto::Acceptor,
            KindDto::Transducer,
            KindDto::Enumerator,
            KindDto::Beaver,
        ] {
            assert_eq!(kind_dto(kind(k)), k);
        }
        for d in [
            DecoderDto::Word,
            DecoderDto::Unary { symbol: "x".into() },
            DecoderDto::Binary {
                order: BitOrderDto::Msb,
            },
            DecoderDto::Binary {
                order: BitOrderDto::Lsb,
            },
        ] {
            assert_eq!(decoder_dto(decoder(&d).unwrap()), d);
        }
    }

    #[test]
    fn diagnostics_map_positions() {
        let source = "δ(q0, ␣) = (q9, x, R)";
        let at = u32::try_from(source.find("q9").unwrap()).unwrap();
        let d = Diagnostic::error("E004", "unknown state")
            .with_span(turing_core::Span::new(at, at + 2))
            .with_help("did you mean `q0`?")
            .with_related(turing_core::Span::new(2, 4), "declared here");
        let dto = diagnostic_dto(&d, &Positions::source(source));
        // `δ(q0, ␣) = (` is 12 UTF-16 code units but 15 bytes.
        assert_eq!(at, 15);
        assert_eq!((dto.from, dto.to), (Some(12), Some(14)));
        assert_eq!((dto.line, dto.column), (Some(1), Some(13)));
        assert_eq!(dto.related[0].from, 1);
        assert_eq!(dto.help.as_deref(), Some("did you mean `q0`?"));
        let passed = diagnostic_dto(&d, &Positions::Utf16);
        assert_eq!((passed.from, passed.to), (Some(at), Some(at + 2)));
        assert_eq!((passed.line, passed.column), (None, None));
    }

    #[test]
    fn counts_saturate() {
        assert_eq!(count(f64::NAN), 0);
        assert_eq!(count(-5.0), 0);
        assert_eq!(count(f64::NEG_INFINITY), 0);
        assert_eq!(count(3.9), 3);
        assert_eq!(count(1e300), MAX_SAFE);
        assert_eq!(count(f64::INFINITY), MAX_SAFE);
        assert_eq!(number(u64::MAX), 9_007_199_254_740_991.0);
        assert_eq!(number(47_176_870), 47_176_870.0);
        assert_eq!(signed(i64::MIN), -9_007_199_254_740_991.0);
        assert_eq!(signed(-3), -3.0);
    }
}
