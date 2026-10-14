# Machines

The gallery of the site and the golden tests of `turing-core` read their machines from this
directory. It is the only source: nothing else lists machines.

* `course/` holds the machines from the course Theoretische Informatik und Logik (TU Dresden,
  summer 2024), rewritten for a tape that is infinite in both directions and for the moves `L`
  and `R` only.
* `classic/` holds textbook examples written for this project.
* `gallery.json` lists every machine with its provenance and expected results. Machines from the
  busy beaver literature are given inline in the bbchallenge standard format.

The `.tm` files use the description language documented in [`docs/dsl.md`](../docs/dsl.md).

## `gallery.json`

```json
{
  "version": 1,
  "groups": [{ "id": "course", "title": "From the course" }],
  "machines": [
    {
      "id": "unary-multiplication",
      "title": "Unary multiplication",
      "group": "course",
      "file": "course/unary-multiplication.tm",
      "provenance": "TheoLog 2024, exercise sheet 1, task 3",
      "source": "theolog2024",
      "description": "",
      "budget": 1000000,
      "examples": [
        { "input": "#xxx#xx#", "expect": { "halts": true, "output": "xxxxxx", "value": "6" } }
      ]
    }
  ]
}
```

| Field | Meaning |
|---|---|
| `id` | Stable identifier, used in share links such as `#g=bb5`. |
| `title` | Display name. |
| `group` | One of the `groups`, in the order shown. |
| `file` | A `.tm` file relative to this directory. Exactly one of `file` and `standard` is present. |
| `standard` | The machine in the bbchallenge standard format. |
| `provenance` | Where the machine comes from, in words. |
| `source` | Bibliography key: `theolog2024` (the course), `original` (written for this project), `bbchallenge` (the bbchallenge wiki), `wolframscience` (the Wolfram 2,3 prize pages). |
| `description` | Prose for the gallery, written separately. |
| `budget` | The largest number of steps a run of an example may take. |
| `examples` | Inputs with their expected results. |

Every example is run from its `input` for at most `budget` steps. The fields of `expect` are
optional:

| Field | Expectation |
|---|---|
| `halts` | Whether the machine halts within the budget. |
| `steps` | The number of steps made until the machine halted. |
| `ones` | The number of non-blank cells when the machine halted. |
| `output` | The output word: the tape from the leftmost to the rightmost non-blank cell. |
| `value` | The output word read as a number by the machine's decoder, in decimal. |
| `accepted` | Whether the machine halted in a final state. |
| `emitted` | The first outputs of an enumerator: every step into a final state outputs the tape, and so does the start if the start state is final. |

`crates/turing-core/tests/gallery.rs` checks every expectation.
