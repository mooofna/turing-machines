# Test data

The CSV files in this directory are the verified enumerations extracted from the Coq-BB5
proofs (mxdys and the bbchallenge collaboration, release v1.0.0, MIT licence, see `NOTICE` in
the repository root). Each row lists one node of the tree normal form in enumeration order
with its status (`halt` or `nonhalt`) and the identifier of the decider that classified it.
`tests/coq_csv.rs` compares the enumeration of this crate with them node by node.

| File | Rows | SHA-256 | Source |
|---|---|---|---|
| `BB2_verified_enumeration.csv` | 61 | `29961b284231d90974e220f3771a8e4ea630cc4822afcd5477f645e964aa909d` | <https://docs.bbchallenge.org/CoqBB5_release_v1.0.0/BB2_verified_enumeration.csv> |
| `BB3_verified_enumeration.csv` | 5,417 | `d1b5df70df506eab6607edeaa181274d5f5a7c58e8aad24bba4a991c351e9c38` | <https://docs.bbchallenge.org/CoqBB5_release_v1.0.0/BB3_verified_enumeration.csv> |
| `BB2x3_verified_enumeration.csv` | 6,019 | `cc33090ebecc2a55ea95b50d66b0d955108e70a14c726b1d1c702040a0d5cd85` | <https://docs.bbchallenge.org/CoqBB5_release_v1.0.0/BB2x3_verified_enumeration.csv> |

These hashes are the ones the extraction scripts of Coq-BB5 check
(`CoqBB5/BB*/BB*_Extraction/BB*_Extraction.sh`). The files have no final newline; keep them
byte for byte as downloaded (`.gitattributes` and `.editorconfig` here prevent conversions).

## BB(4)

The BB(4) enumeration (858,909 rows, 45 MB) is not committed. `tests/coq_bb4.rs` reads it from
the path in the environment variable `COQ_BB4_CSV` and otherwise skips the comparison.

* Source: <https://docs.bbchallenge.org/CoqBB5_release_v1.0.0/BB4_verified_enumeration.csv>
* SHA-256 of the file as served (no final newline):
  `c517de277a3f0c8c95ffefa510caf834d8f53763e59623813f98c2b64c6df621`
* SHA-256 with a final newline appended, which is the value pinned in Coq-BB5's
  `BB4_Extraction.sh`: `6a4abba432b1b36ad7c45a2382ed1d8ce08aee30401964e36ce74453755b5c08`

```sh
curl -LO https://docs.bbchallenge.org/CoqBB5_release_v1.0.0/BB4_verified_enumeration.csv
shasum -a 256 BB4_verified_enumeration.csv
COQ_BB4_CSV=$PWD/BB4_verified_enumeration.csv cargo test --release -p turing-bb --test coq_bb4
```
