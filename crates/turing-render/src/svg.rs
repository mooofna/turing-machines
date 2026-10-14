//! A complete state diagram as SVG, for the command-line tool and the README.

use crate::graph::Graph;
use crate::layout::Layout;
use crate::route::EdgeRoute;

/// Writes a standalone SVG with course-style states (circles, final states doubled, an arrow
/// into the start state) in the given ink and accent colours.
pub fn to_svg(
    graph: &Graph,
    layout: &Layout,
    routes: &[EdgeRoute],
    ink: &str,
    background: &str,
) -> String {
    let _ = (graph, layout, routes, ink, background);
    todo!("A4")
}
