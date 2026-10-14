//! The canonical formatter.

use alloc::string::String;

use super::FormatStyle;
use crate::spec::MachineSpec;

pub(super) fn format(spec: &MachineSpec, style: FormatStyle) -> String {
    let _ = (spec, style);
    todo!("A2")
}
