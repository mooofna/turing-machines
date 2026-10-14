//! Share links without a server.
//!
//! A payload is `1.` followed by the base64url encoding (without padding) of the raw DEFLATE
//! compression (level 9) of the canonical description in Unicode style:
//!
//! ```text
//! "1." + base64url_nopad(deflate_raw(format(spec, Unicode)))
//! ```
//!
//! The leading `1` is the format version. Decoding never fails with an exception: every problem
//! becomes an `E100` diagnostic with a plain message. The decompressed text is limited to
//! [`MAX_TEXT`] bytes, so a crafted link cannot exhaust memory, and links of larger machines
//! are refused when encoding.

use base64::Engine as _;
use base64::engine::general_purpose::{URL_SAFE_NO_PAD, URL_SAFE_NO_PAD_INDIFFERENT};
use miniz_oxide::inflate::TINFLStatus;
use turing_core::dsl::{self, FormatStyle};

use crate::Error;
use crate::convert;
use crate::dto::{DiagnosticDto, MachineSpecDto, ParseResultDto, SeverityDto};
use crate::editor;

/// The diagnostic code of every share link problem.
pub(crate) const CODE: &str = "E100";

/// The version prefix of the current format.
pub(crate) const PREFIX: &str = "1.";

/// The largest description a link may carry, in bytes of UTF-8.
pub(crate) const MAX_TEXT: usize = 64 * 1024;

/// The largest payload accepted for decoding, in bytes. Deflate never expands its input by
/// more than a few bytes per 64 KB block, and base64 adds a third.
const MAX_PAYLOAD: usize = 96 * 1024;

/// Encodes a spec for a link.
///
/// # Errors
///
/// Fails for a malformed glyph or a description larger than [`MAX_TEXT`].
pub(crate) fn encode(dto: &MachineSpecDto) -> Result<String, Error> {
    let spec = convert::spec(dto).map_err(|p| convert::first_error(&p))?;
    let text = dsl::format(&spec, FormatStyle::Unicode);
    if text.len() > MAX_TEXT {
        return Err(Error::new(format!(
            "the description has {} bytes, but a link holds at most {MAX_TEXT}",
            text.len()
        )));
    }
    Ok(pack(&text))
}

/// The payload of `text`.
pub(crate) fn pack(text: &str) -> String {
    let deflated = miniz_oxide::deflate::compress_to_vec(text.as_bytes(), 9);
    let mut out = String::with_capacity(PREFIX.len() + deflated.len().div_ceil(3) * 4);
    out.push_str(PREFIX);
    URL_SAFE_NO_PAD.encode_string(&deflated, &mut out);
    out
}

/// The description in `payload`.
///
/// # Errors
///
/// A plain English message for every way a payload can be wrong.
pub(crate) fn unpack(payload: &str) -> Result<String, String> {
    let payload = payload.trim();
    let Some(body) = payload.strip_prefix(PREFIX) else {
        let newer = payload
            .split_once('.')
            .is_some_and(|(v, _)| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()));
        return Err(if newer {
            "This link was made by a newer version of the site and cannot be opened here.".into()
        } else {
            "This is not a link to a shared machine.".into()
        });
    };
    if body.len() > MAX_PAYLOAD {
        return Err("The shared machine is too large.".into());
    }
    let deflated = URL_SAFE_NO_PAD_INDIFFERENT
        .decode(body)
        .map_err(|_| "The link is damaged: it contains characters that do not belong there.")?;
    let bytes = miniz_oxide::inflate::decompress_to_vec_with_limit(&deflated, MAX_TEXT).map_err(
        |e| match e.status {
            TINFLStatus::HasMoreOutput => {
                format!("The shared machine is larger than {} KB.", MAX_TEXT / 1024)
            }
            _ => "The link is damaged: its data cannot be unpacked.".into(),
        },
    )?;
    String::from_utf8(bytes).map_err(|_| "The link is damaged: its text is not valid UTF-8.".into())
}

/// Decodes a payload and parses the description in it. `source` is set whenever the payload
/// could be unpacked, so a description with errors can still be opened in the text editor.
pub(crate) fn decode(payload: &str) -> ParseResultDto {
    match unpack(payload) {
        Ok(text) => {
            let mut result = editor::parse_source(&text);
            result.source = Some(text);
            result
        }
        Err(message) => ParseResultDto {
            spec: None,
            diagnostics: vec![DiagnosticDto {
                severity: SeverityDto::Error,
                code: CODE.into(),
                message,
                help: None,
                from: None,
                to: None,
                line: None,
                column: None,
                related: Vec::new(),
            }],
            source: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "machine \"increment\"\nΣ = {0, 1}\n\nδ(q0, 1) = (q0, 0, R)\nδ(q0, 0) = (q1, 1, R)\nδ(q0, ␣) = (q1, 1, R)\n";

    #[test]
    fn pack_round_trips() {
        let payload = pack(SOURCE);
        assert!(payload.starts_with("1."));
        assert!(
            payload[2..]
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
            "{payload}"
        );
        assert_eq!(unpack(&payload).as_deref(), Ok(SOURCE));
        // Surrounding white space and padding are tolerated.
        let padding = "=".repeat((4 - (payload.len() - 2) % 4) % 4);
        assert_eq!(
            unpack(&format!("  {payload}{padding}\n")).as_deref(),
            Ok(SOURCE)
        );
    }

    #[test]
    fn empty_text_round_trips() {
        assert_eq!(unpack(&pack("")).as_deref(), Ok(""));
    }

    #[test]
    fn compression_pays_off() {
        let long = SOURCE.repeat(20);
        assert!(pack(&long).len() < long.len() / 4);
    }

    #[test]
    fn versions_are_checked() {
        let payload = pack(SOURCE);
        let newer = payload.replacen("1.", "2.", 1);
        assert!(unpack(&newer).unwrap_err().contains("newer version"));
        assert!(unpack("hello").unwrap_err().contains("not a link"));
        assert!(unpack("").unwrap_err().contains("not a link"));
        assert!(unpack(".abc").unwrap_err().contains("not a link"));
    }

    #[test]
    fn damage_is_reported() {
        assert!(unpack("1.@@@").unwrap_err().contains("characters"));
        assert!(
            unpack("1.AAAAAAAA")
                .unwrap_err()
                .contains("cannot be unpacked")
        );
        let not_utf8 = format!(
            "1.{}",
            URL_SAFE_NO_PAD.encode(miniz_oxide::deflate::compress_to_vec(&[0xff, 0xfe], 9))
        );
        assert!(unpack(&not_utf8).unwrap_err().contains("UTF-8"));
        // A truncated payload.
        let payload = pack(SOURCE);
        assert!(unpack(&payload[..payload.len() / 2]).is_err());
    }

    #[test]
    fn bombs_are_refused() {
        // 10 MB of zeros deflates to about 10 KB.
        let zeros = vec![b'0'; 10 * 1024 * 1024];
        let bomb = format!(
            "1.{}",
            URL_SAFE_NO_PAD.encode(miniz_oxide::deflate::compress_to_vec(&zeros, 9))
        );
        assert!(bomb.len() < MAX_PAYLOAD);
        assert!(unpack(&bomb).unwrap_err().contains("larger than 64 KB"));
        let huge = format!("1.{}", "A".repeat(MAX_PAYLOAD + 1));
        assert!(unpack(&huge).unwrap_err().contains("too large"));
    }

    #[test]
    fn decode_errors_are_diagnostics() {
        let result = decode("garbage");
        assert_eq!(result.spec, None);
        assert_eq!(result.source, None);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, CODE);
        assert_eq!(result.diagnostics[0].severity, SeverityDto::Error);
        assert_eq!(result.diagnostics[0].from, None);
    }
}

#[cfg(test)]
mod round_trips {
    use super::*;
    use crate::testing;

    #[test]
    fn specs_round_trip() {
        for spec in [
            testing::increment(),
            testing::tally_enumerator(),
            testing::standard(testing::BB5),
        ] {
            let payload = encode(&spec).unwrap();
            let decoded = decode(&payload);
            assert_eq!(decoded.diagnostics, []);
            let text = editor::format_spec(&spec, false).unwrap();
            assert_eq!(decoded.source.as_deref(), Some(text.as_str()));
            assert_eq!(decoded.spec, editor::parse_source(&text).spec);
        }
    }

    #[test]
    fn sources_with_errors_are_kept() {
        let payload = pack("δ(q0, ␣) = (q1, x, R)\nδ(q0, ␣) = (q2, x, R)\n");
        let decoded = decode(&payload);
        assert!(decoded.spec.is_none());
        assert_ne!(decoded.diagnostics.len(), 0);
        assert!(decoded.source.is_some());
    }

    #[test]
    fn large_machines_are_refused() {
        let mut spec = testing::increment();
        spec.comments.header = vec!["x".repeat(MAX_TEXT)];
        assert!(encode(&spec).is_err());
    }
}
