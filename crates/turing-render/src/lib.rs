#![doc = include_str!("../README.md")]

pub mod graph;
pub mod layout;
pub mod palette;
pub mod route;
pub mod spacetime;
pub mod svg;

pub use graph::{Edge, Graph, Node};
pub use layout::{Layout, LayoutKind, layout};
pub use palette::{Palette, Rgb};
pub use route::{EdgeRoute, route};
pub use spacetime::{
    CellRef, SpaceTime, SpaceTimeError, SpaceTimeMode, SpaceTimeProgress, SpaceTimeRequest,
};
