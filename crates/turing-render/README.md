# turing-render

Pictures of Turing machines, computed without any graphics dependency.

* `spacetime` renders a run as an image: time runs downwards, the tape runs across, every pixel
  is a cell or a bin of cells. Rendering first fills an index buffer (one byte per pixel) and
  colours it separately, so a theme change does not re-run the machine. Long runs are sampled
  exactly with the chain simulator of `turing-core`.
* `layout` places the states of a state diagram, layered from the start state for sparse
  machines and on a circle for dense ones.
* `route` draws the edges: straight or curved, opposite pairs apart, self-loops where there is
  room, one stacked label per pair of states.
* `svg` writes a complete state diagram as SVG.
