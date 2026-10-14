//! Tree normal form: the nodes of the enumeration tree.

use crate::compact::CompactTm;

/// A node of the tree normal form: a partial machine.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Node {
    /// The partial machine.
    pub(crate) tm: CompactTm,
    /// The depth in the tree, 0 for the root.
    pub(crate) depth: u8,
}
