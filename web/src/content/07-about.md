# About

This site is written and maintained by Maurice Heide (MSc Data Science, ETH Zurich; BSc Computer Science, TU Dresden). More about the author is on his [homepage](https://mooofna.github.io), and the source code is on [GitHub](https://github.com/mooofna/turing-machines).

## Origin

The project began as a course exercise. In the summer semester 2024, the course Theoretische Informatik und Logik at TU Dresden, taught by Prof. Markus Krötzsch and Dr. Stephan Mennicke, asked on its first exercise sheet for a Turing machine $A_{\mathrm{mul}}$ that multiplies two numbers in unary, and for a program that reads two decimal numbers, encodes them in unary, runs the machine and decodes the result [@theolog2024]. The same sheet asked for the doubling machine $M_{*2}$, which lecture 2 of the course uses in the proof that the busy beaver function is not computable, in a multi-tape and a single-tape version.

The 2024 solution was a single Rust file: a small simulator that read machines from JSON files and printed their configurations. In 2026 it was rewritten as a Rust workspace with a WebAssembly build, and the exercise machines became the starting point of the chapters on this site.

## What is in the repository

The repository is a Rust workspace with the following crates.

- `turing-core`: the simulator and the language for describing machines.
- `turing-bb`: the enumeration in tree normal form and the deciders of chapter 5.
- `turing-render`: diagrams of machines and of their runs.
- `turing-cli`: the command-line tool `tm`.
- `turing-wasm`: the WebAssembly bindings used by the web front end.

The web front end is written with SvelteKit.

## Privacy

The site sets no cookies and does not track visitors. It sends no requests to other servers: every machine on this site is simulated in your browser, and nothing you enter leaves your device.

## Sources

The numerical facts quoted in the chapters are either computed by the page while you read it or taken from a ledger in which every fact has a source. The bibliography below is generated from that ledger.

{{bibliography}}

## Licence

The code is available under the MIT licence or the Apache License 2.0, at your option (`MIT OR Apache-2.0`).

## How to cite

If you refer to this site, please cite it as follows.

```bibtex
@misc{heide-turing-machines,
  author       = {Maurice Heide},
  title        = {Turing machines},
  howpublished = {\url{https://mooofna.github.io/turing-machines/}},
  note         = {Interactive simulator and notes, source code at \url{https://github.com/mooofna/turing-machines}},
  year         = {2026}
}
```

## Acknowledgements

The tests of this project compare the enumeration and the deciders on this site with the formally verified enumeration of the Coq-BB5 project [@coqbb5], whose data are published under the MIT licence. I thank the bbchallenge collaboration for making this data openly available. The exercise tasks described on this site are taken from the course materials of Theoretische Informatik und Logik [@theolog2024].
