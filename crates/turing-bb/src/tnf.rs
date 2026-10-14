//! Tree normal form: the nodes of the enumeration tree.
//!
//! The tree normal form (TNF) enumerates partial machines instead of complete ones. The root is
//! the machine with every transition undefined. A node is classified by the decider pipeline:
//!
//! * If it reaches an undefined transition δ(q, a) from the blank tape after t steps, it is a
//!   *halting node*. Completing δ(q, a) with a transition to the halting state gives a machine
//!   that halts after t + 1 steps. If more than one transition is undefined, the node gets one
//!   child for every way to define δ(q, a): every symbol, both moves and every next state up
//!   to the first state not yet entered by any transition (states are introduced in order, so
//!   machines that differ only by a renaming of states appear once). At the root only moves to
//!   the right are used, because a machine whose first move is to the left is the mirror image
//!   of one whose first move is to the right.
//! * If it is proved never to halt, it is a leaf, and so are all machines that agree with it
//!   on its defined transitions, since they run identically.
//!
//! Every machine that halts from the blank tape behaves, up to renaming and mirroring, like a
//! halting node until it halts, with the halting transition in place of the undefined one: the
//! node consisting of exactly the transitions it uses before halting is reached by the
//! enumeration, because each of its ancestors reaches an undefined transition no later than
//! the machine halts and is therefore expanded. So if every node is classified, the busy
//! beaver values are the maxima over the halting nodes.
//!
//! This is the enumeration of Coq-BB5 (`TNF.v`, `BB4_TNF_Enumeration.v`), including its order:
//! depth first, children in the order (next state, move L before R, symbol written). Unlike
//! the variant `TNF-1RB`, it keeps both `0RB` and `1RB` (and `0RA`, `1RA`) as first
//! transitions.

use crate::compact::{CompactAction, CompactTm, Next};
use turing_core::Dir;

/// A node of the tree normal form: a partial machine.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Node {
    /// The partial machine.
    pub(crate) tm: CompactTm,
    /// The depth in the tree, 0 for the root.
    pub(crate) depth: u8,
}

impl Node {
    /// The root: every transition undefined.
    pub(crate) fn root(states: u8, symbols: u8) -> Self {
        Self {
            tm: CompactTm::new(states, symbols),
            depth: 0,
        }
    }

    /// Pushes the children of a halting node that stopped at the undefined transition `at`
    /// onto a depth-first stack, so that the first child in Coq-BB5's order is popped first.
    /// Pushes nothing if `at` is the last undefined transition.
    pub(crate) fn push_children(&self, at: (u8, u8), stack: &mut Vec<Self>) {
        if self.tm.undefined() <= 1 {
            return;
        }
        let bound = self.tm.next_state_bound();
        let dirs: &[Dir] = if self.depth == 0 {
            &[Dir::R]
        } else {
            &[Dir::L, Dir::R]
        };
        let first = stack.len();
        for next in 0..=bound {
            for &dir in dirs {
                for write in 0..self.tm.symbols {
                    let mut tm = self.tm;
                    tm.set(
                        at.0,
                        at.1,
                        Some(CompactAction {
                            write,
                            dir,
                            next: Next::State(next),
                        }),
                    );
                    stack.push(Self {
                        tm,
                        depth: self.depth + 1,
                    });
                }
            }
        }
        stack[first..].reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn children(node: &Node, at: (u8, u8)) -> Vec<String> {
        let mut stack = Vec::new();
        node.push_children(at, &mut stack);
        stack.iter().rev().map(|n| n.tm.to_string()).collect()
    }

    #[test]
    fn the_root_moves_right_only() {
        let root = Node::root(2, 2);
        assert_eq!(
            children(&root, (0, 0)),
            [
                "0RA---_------",
                "1RA---_------",
                "0RB---_------",
                "1RB---_------"
            ]
        );
    }

    #[test]
    fn new_states_are_introduced_in_order() {
        let node = Node {
            tm: "0RB---_------_------".parse().unwrap(),
            depth: 1,
        };
        let kids = children(&node, (1, 0));
        assert_eq!(kids.len(), 12);
        assert_eq!(kids[0], "0RB---_0LA---_------");
        assert_eq!(kids[11], "0RB---_1RC---_------");
    }

    #[test]
    fn the_last_undefined_transition_is_not_expanded() {
        let node = Node {
            tm: "1RB1LB_1LA---".parse().unwrap(),
            depth: 3,
        };
        assert_eq!(children(&node, (1, 1)), Vec::<String>::new());
    }
}
