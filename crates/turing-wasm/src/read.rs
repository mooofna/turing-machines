//! Reading the input DTOs from JavaScript values.
//!
//! The DTOs derive `Deserialize`, and their TypeScript types come from the same definitions,
//! but deserializing through serde instantiates a visitor per type and buffers internally
//! tagged enums, which costs a large share of the module size. These readers accept exactly
//! the values that the generated TypeScript types describe, in a fraction of the code, and
//! name the offending field in their errors, for example
//! `spec.transitions[2].read[0] must be a string`.
//!
//! As with serde: properties that are not part of a type are ignored, optional properties may
//! be missing, `undefined` or `null`, and integers must be whole numbers in range.

use std::fmt;

use js_sys::{Array, Reflect};
use wasm_bindgen::{JsCast, JsValue};

use crate::Error;
use crate::dto::{
    BitOrderDto, BreakpointsDto, CommentsDto, DeclaredDto, DecoderDto, DirDto, KindDto,
    MachineSpecDto, PaletteDto, SpaceTimeModeDto, SpaceTimeRequestDto, SpanDto, StateDto,
    TransitionDto,
};

/// Where a value sits in its argument, for error messages; built lazily.
#[derive(Copy, Clone)]
enum Path<'a> {
    Root(&'a str),
    Key(&'a Path<'a>, &'a str),
    Index(&'a Path<'a>, usize),
}

impl fmt::Display for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Root(name) => f.write_str(name),
            Self::Key(parent, key) => write!(f, "{parent}.{key}"),
            Self::Index(parent, i) => write!(f, "{parent}[{i}]"),
        }
    }
}

impl<'a> Path<'a> {
    fn key(&'a self, key: &'a str) -> Self {
        Self::Key(self, key)
    }

    fn index(&'a self, i: usize) -> Self {
        Self::Index(self, i)
    }

    fn error(&self, expected: &str) -> Error {
        Error::new(format!("{self} must be {expected}"))
    }
}

/// An object whose properties are read one by one.
struct Object<'a> {
    value: &'a JsValue,
    path: Path<'a>,
}

impl<'a> Object<'a> {
    fn new(value: &'a JsValue, path: Path<'a>) -> Result<Self, Error> {
        if value.is_object() && !Array::is_array(value) {
            Ok(Self { value, path })
        } else {
            Err(path.error("an object"))
        }
    }

    /// The property `key`; `undefined` if it is missing.
    fn get(&self, key: &JsValue) -> JsValue {
        Reflect::get(self.value, key).unwrap_or(JsValue::UNDEFINED)
    }

    fn string(&self, key: &str) -> Result<String, Error> {
        string(&self.get(&JsValue::from_str(key)), &self.path.key(key))
    }

    fn bool(&self, key: &str) -> Result<bool, Error> {
        self.get(&JsValue::from_str(key))
            .as_bool()
            .ok_or_else(|| self.path.key(key).error("a boolean"))
    }

    fn number(&self, key: &str) -> Result<f64, Error> {
        self.get(&JsValue::from_str(key))
            .as_f64()
            .ok_or_else(|| self.path.key(key).error("a number"))
    }

    fn integer(&self, key: &str, max: u32) -> Result<u32, Error> {
        integer(&self.get(&JsValue::from_str(key)), &self.path.key(key), max)
    }

    fn strings(&self, key: &str) -> Result<Vec<String>, Error> {
        let path = self.path.key(key);
        list(&self.get(&JsValue::from_str(key)), &path, |v, p| {
            string(v, &p)
        })
    }
}

fn string(value: &JsValue, path: &Path<'_>) -> Result<String, Error> {
    value.as_string().ok_or_else(|| path.error("a string"))
}

/// A whole number from 0 to `max`.
fn integer(value: &JsValue, path: &Path<'_>, max: u32) -> Result<u32, Error> {
    value
        .as_f64()
        .filter(|n| n.fract() == 0.0 && *n >= 0.0 && *n <= f64::from(max))
        .map(|n| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "checked to be a whole number in 0..=max"
            )]
            let n = n as u32;
            n
        })
        .ok_or_else(|| path.error(&format!("a whole number from 0 to {max}")))
}

/// The elements of an array, each read with `item`.
fn list<T>(
    value: &JsValue,
    path: &Path<'_>,
    mut item: impl FnMut(&JsValue, Path<'_>) -> Result<T, Error>,
) -> Result<Vec<T>, Error> {
    if !Array::is_array(value) {
        return Err(path.error("an array"));
    }
    let array: &Array = value.unchecked_ref();
    (0..array.length())
        .map(|i| {
            let index = usize::try_from(i).unwrap_or(usize::MAX);
            item(&array.get(i), path.index(index))
        })
        .collect()
}

/// One of the string values of an enum.
fn choice<T: Copy>(value: &JsValue, path: &Path<'_>, options: &[(&str, T)]) -> Result<T, Error> {
    let text = value.as_string();
    options
        .iter()
        .find(|(name, _)| text.as_deref() == Some(*name))
        .map(|(_, v)| *v)
        .ok_or_else(|| {
            let names: Vec<String> = options.iter().map(|(n, _)| format!("`{n}`")).collect();
            path.error(&format!("one of {}", names.join(", ")))
        })
}

/// The property names of a transition, created once per spec instead of once per access.
struct TransitionKeys {
    from: JsValue,
    read: JsValue,
    to: JsValue,
    write: JsValue,
    dir: JsValue,
    comment: JsValue,
    leading: JsValue,
    span: JsValue,
}

impl TransitionKeys {
    fn new() -> Self {
        Self {
            from: JsValue::from_str("from"),
            read: JsValue::from_str("read"),
            to: JsValue::from_str("to"),
            write: JsValue::from_str("write"),
            dir: JsValue::from_str("dir"),
            comment: JsValue::from_str("comment"),
            leading: JsValue::from_str("leading"),
            span: JsValue::from_str("span"),
        }
    }
}

const DIRS: [(&str, DirDto); 3] = [("L", DirDto::L), ("R", DirDto::R), ("N", DirDto::N)];

fn transition(
    value: &JsValue,
    path: Path<'_>,
    keys: &TransitionKeys,
) -> Result<TransitionDto, Error> {
    let o = Object::new(value, path)?;
    let strings =
        |key: &JsValue, name: &str| list(&o.get(key), &o.path.key(name), |v, p| string(v, &p));
    let comment = o.get(&keys.comment);
    let leading = o.get(&keys.leading);
    let span = o.get(&keys.span);
    Ok(TransitionDto {
        from: string(&o.get(&keys.from), &o.path.key("from"))?,
        read: strings(&keys.read, "read")?,
        to: string(&o.get(&keys.to), &o.path.key("to"))?,
        write: strings(&keys.write, "write")?,
        dir: list(&o.get(&keys.dir), &o.path.key("dir"), |v, p| {
            choice(v, &p, &DIRS)
        })?,
        comment: if comment.is_undefined() || comment.is_null() {
            None
        } else {
            Some(string(&comment, &o.path.key("comment"))?)
        },
        leading: if leading.is_undefined() {
            Vec::new()
        } else {
            list(&leading, &o.path.key("leading"), |v, p| string(v, &p))?
        },
        span: if span.is_undefined() || span.is_null() {
            None
        } else {
            let s = Object::new(&span, o.path.key("span"))?;
            Some(SpanDto {
                from: s.integer("from", u32::MAX)?,
                to: s.integer("to", u32::MAX)?,
            })
        },
    })
}

fn decoder(value: &JsValue, path: Path<'_>) -> Result<DecoderDto, Error> {
    let o = Object::new(value, path)?;
    let kind = choice(
        &o.get(&JsValue::from_str("kind")),
        &o.path.key("kind"),
        &[("word", 0), ("unary", 1), ("binary", 2)],
    )?;
    Ok(match kind {
        0 => DecoderDto::Word,
        1 => DecoderDto::Unary {
            symbol: o.string("symbol")?,
        },
        _ => DecoderDto::Binary {
            order: choice(
                &o.get(&JsValue::from_str("order")),
                &o.path.key("order"),
                &[("lsb", BitOrderDto::Lsb), ("msb", BitOrderDto::Msb)],
            )?,
        },
    })
}

/// Reads a `MachineSpecDto`.
///
/// # Errors
///
/// Names the first property that does not match the type.
pub(crate) fn spec(value: &JsValue) -> Result<MachineSpecDto, Error> {
    let o = Object::new(value, Path::Root("spec"))?;
    let keys = TransitionKeys::new();
    let declared = o.get(&JsValue::from_str("declared"));
    let declared = Object::new(&declared, o.path.key("declared"))?;
    let comments = o.get(&JsValue::from_str("comments"));
    let comments = Object::new(&comments, o.path.key("comments"))?;
    let tapes = o.integer("tapes", u32::from(u8::MAX))?;
    Ok(MachineSpecDto {
        name: o.string("name")?,
        kind: choice(
            &o.get(&JsValue::from_str("kind")),
            &o.path.key("kind"),
            &[
                ("acceptor", KindDto::Acceptor),
                ("transducer", KindDto::Transducer),
                ("enumerator", KindDto::Enumerator),
                ("beaver", KindDto::Beaver),
            ],
        )?,
        tapes: u8::try_from(tapes).unwrap_or(u8::MAX),
        blank: o.string("blank")?,
        input_alphabet: o.strings("inputAlphabet")?,
        tape_alphabet: o.strings("tapeAlphabet")?,
        states: list(
            &o.get(&JsValue::from_str("states")),
            &o.path.key("states"),
            |v, p| {
                let s = Object::new(v, p)?;
                Ok(StateDto {
                    name: s.string("name")?,
                    is_final: s.bool("isFinal")?,
                })
            },
        )?,
        start: o.string("start")?,
        transitions: list(
            &o.get(&JsValue::from_str("transitions")),
            &o.path.key("transitions"),
            |v, p| transition(v, p, &keys),
        )?,
        input: o.string("input")?,
        output: decoder(&o.get(&JsValue::from_str("output")), o.path.key("output"))?,
        declared: DeclaredDto {
            states: declared.bool("states")?,
            input_alphabet: declared.bool("inputAlphabet")?,
            tape_alphabet: declared.bool("tapeAlphabet")?,
            start: declared.bool("start")?,
            blank: declared.bool("blank")?,
        },
        comments: CommentsDto {
            header: comments.strings("header")?,
            trailing: comments.strings("trailing")?,
        },
    })
}

/// Reads a `BreakpointsDto`.
///
/// # Errors
///
/// Names the first property that does not match the type.
pub(crate) fn breakpoints(value: &JsValue) -> Result<BreakpointsDto, Error> {
    let o = Object::new(value, Path::Root("breakpoints"))?;
    let on_output = o.get(&JsValue::from_str("onOutput"));
    Ok(BreakpointsDto {
        transitions: list(
            &o.get(&JsValue::from_str("transitions")),
            &o.path.key("transitions"),
            |v, p| integer(v, &p, u32::MAX).map(|i| usize::try_from(i).unwrap_or(usize::MAX)),
        )?,
        states: o.strings("states")?,
        on_output: if on_output.is_undefined() {
            false
        } else {
            on_output
                .as_bool()
                .ok_or_else(|| o.path.key("onOutput").error("a boolean"))?
        },
    })
}

/// Reads a `SpaceTimeRequestDto`.
///
/// # Errors
///
/// Names the first property that does not match the type.
pub(crate) fn spacetime_request(value: &JsValue) -> Result<SpaceTimeRequestDto, Error> {
    let o = Object::new(value, Path::Root("request"))?;
    let mode = o.get(&JsValue::from_str("mode"));
    let m = Object::new(&mode, o.path.key("mode"))?;
    let kind = choice(
        &m.get(&JsValue::from_str("kind")),
        &m.path.key("kind"),
        &[("exact", true), ("overview", false)],
    )?;
    let mode = if kind {
        let cell_px = m.get(&JsValue::from_str("cellPx"));
        SpaceTimeModeDto::Exact {
            first_step: m.number("firstStep")?,
            cell_px: if cell_px.is_undefined() || cell_px.is_null() {
                None
            } else {
                Some(integer(&cell_px, &m.path.key("cellPx"), u32::MAX)?)
            },
        }
    } else {
        SpaceTimeModeDto::Overview {
            max_steps: m.number("maxSteps")?,
        }
    };
    Ok(SpaceTimeRequestDto {
        width: o.integer("width", u32::MAX)?,
        height: o.integer("height", u32::MAX)?,
        mode,
    })
}

/// Reads a `PaletteDto`.
///
/// # Errors
///
/// Names the first property that does not match the type.
pub(crate) fn palette(value: &JsValue) -> Result<PaletteDto, Error> {
    let o = Object::new(value, Path::Root("palette"))?;
    Ok(PaletteDto {
        background: o.string("background")?,
        ink: o.string("ink")?,
        accent: o.string("accent")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_name_nested_fields() {
        let root = Path::Root("spec");
        let transitions = root.key("transitions");
        let third = transitions.index(2);
        let read = third.key("read");
        let first = read.index(0);
        assert_eq!(
            first.error("a string").to_string(),
            "spec.transitions[2].read[0] must be a string"
        );
    }
}
