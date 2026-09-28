# 🥷 The Binary Ninja lifter

`oinkie lift` drives Binary Ninja headless and reads back one of its three
intermediate languages.

```sh
oinkie lift -r binary-ninja-mlil path/to/binary
```

`-r` names the representation, not the tool, because Binary Ninja produces
three and "binary-ninja" would not say which. The three are
`binary-ninja-llil`, `binary-ninja-mlil` and `binary-ninja-hlil`.

## 📍 Finding the installation

`--home`, then `BINARY_NINJA_HOME`, then the usual locations. What is wanted
is the directory holding `bnpython3` — Binary Ninja's own interpreter, which
has the API importable already, so nothing here touches your `PYTHONPATH` or
your `python3`.

On macOS that is `Contents/MacOS` inside the application bundle; on Linux it
is the installation directory itself. The error names the variable and the
places it looked.

## 🧬 What the three levels give you

They are different vocabularies, not different amounts of detail of one
vocabulary, and oinkie refuses to compare birthmarks across them. Which one
answers your question is yours to decide; what follows is what the shapes
look like, from one `hello world` compiled by clang for arm64:

| level | instructions in `_main` | a call reads |
| --- | --- | --- |
| LLIL | 12 | `LLIL_CALL` with the target address |
| MLIL | 3 | `MLIL_CALL` with outputs, target, parameters |
| HLIL | 2 | `HLIL_CALL` with the callee's name |

The symbol table in the output is keyed by whatever the level renders a call
target as. LLIL and MLIL give an address, so the keys are addresses; HLIL has
already resolved the name, so at that level a key is the name and the entry
is an identity. The Oinkie IR does not require a key to be an address — it
requires the lifter that produced both sides to agree with itself.

## ✍️ Replacing the script

`--script` takes a Python file in place of the built-in one. It is run as

```
bnpython3 <your script> <binary> <level> <output name>
```

with the working directory set to where the output belongs, and it must write
that file as valid Oinkie IR. Two obligations are worth stating, because
breaking either produces something that looks like it worked:

- **Build the JSON with a serialiser.** `json.dumps` — never string
  concatenation. The Ghidra script did the latter and wrote files that could
  not be parsed once a symbol held a quote, and files that parsed into the
  *wrong name* once one held a backslash (#77). `oinkie lift` reads its own
  output back before reporting success, so the first kind now fails loudly;
  the second still parses.
- **Key the symbol table the way your operations render a callee**, since
  that is what the reader looks up. A table nothing matches leaves every
  `fc-*` birthmark empty, and two empty birthmarks score as a perfect match.

## 🧪 Testing

Binary Ninja is licensed and cannot be installed on a CI runner, so nothing
in CI lifts with it — see [the note on all the lifters](../README.md) for why
that is accepted rather than worked around. Two things stand in for it here:

- `python3 -m py_compile` on this script, on every push;
- fixtures under `testdata/lifted/bnil_*/`, lifted on a machine that has
  Binary Ninja and committed, which `tests/binaryninja_test.rs` reads back.
  They carry a repository-relative `path`, edited after lifting the way
  `pcodes/` already is, because a fresh lift records the absolute path it
  canonicalised.

The test that matters most asserts that each level *finds* the call in the
fixture. An `is_call` recognising none of its own opcodes is the failure that
would otherwise report unrelated programs as identical.
