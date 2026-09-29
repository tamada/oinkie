# 🔬 The IDA Pro lifter

`oinkie lift` drives `idat` headless and reads back the Hex-Rays microcode.

```sh
oinkie lift -r ida-microcode-lvars path/to/binary
```

## ☁️ Read this first: the decompiler may be in the cloud

Microcode *is* the decompiler's own representation, so producing it runs the
decompiler. **A cloud decompiler sends each function to Hex-Rays' servers.**
For a tool whose purpose is deciding whether a binary was stolen, the binaries
it is pointed at are the ones whose owner is least likely to want them leaving
the machine.

`oinkie lift` warns before it starts — before, because a warning that arrives
once the function has left is a log entry. It does not prompt and does not
refuse: `lift` is a batch command called from scripts, a prompt would hang a
non-interactive run, and asking for `-r ida-microcode-*` is the consent, since
IDA has no other representation to ask for.

The warning is conditional, and this is how it decides. A decompiler plugin is
named `hex<arch>` when it decompiles locally and `hexc<arch>` when it calls
out, so an installation carrying only `hexc*` has no local decompiler to use.
IDA Classroom ships the cloud ones; a Professional licence with the decompiler
ships local ones.

The API cannot answer this. `get_hexrays_version` reports a version and nothing
about where the work happens, and `MERR_CLOUD` is an error code — it says a
decompilation *already failed* for a cloud reason, which is after the fact.

**That naming convention is not documented anywhere citable.** If it is wrong,
the failure is a missing warning rather than a false one, which is the way round
it should fail: telling someone with a local licence that their functions are
being uploaded is a claim about their data that is not true. The warning names
the plugins it saw, so a wrong inference is legible.

## 📍 Finding the installation

`--home`, then `IDA_HOME`, then the usual locations. What is wanted is the
directory holding `idat`; on macOS that is `Contents/MacOS` inside the
application bundle. The error names the variable and where it looked.

`idalib`, IDA 9's headless library, is deliberately not used: activating it
runs `py-activate-idalib.py`, which rewrites the Python environment oinkie is
running in. `idat` needs nothing installed.

## 🧬 Eight maturities

The decompiler rewrites the microcode through a pipeline, and each stage has
its own vocabulary. All eight are offered; which one answers your question is
yours to decide.

| `--ir` | stage |
| --- | --- |
| `ida-microcode-generated` | as lifted, before optimisation |
| `ida-microcode-preoptimized` | after preoptimisation |
| `ida-microcode-locopt` | after local optimisation |
| `ida-microcode-calls` | calls resolved into calls with arguments |
| `ida-microcode-glbopt1` … `glbopt3` | the three global optimisation passes |
| `ida-microcode-lvars` | local variables allocated; what the pseudocode is rendered from |

`MMAT_ZERO` is not among them: it is the state before any microcode exists, so
there is nothing to write.

They are not copies of one another, and the difference is not only in size. On
one `hello world`, `_main` holds 8 instructions at `preoptimized` and 2 at
`glbopt1` — and at `calls` the call has been folded into the operand of the
instruction that consumes its result, so it appears as a sub-instruction rather
than in the block's own list. The script walks into operands for exactly that
reason; without it that maturity reports no call at all.

## 🔑 The symbol table

Keyed by the callee as the microcode renders it, which is `$name` for a global.
The `$` stays on in the key, because that is what the reader looks up, and comes
off to resolve the name.

Getting that backwards is not loud. `m_call` is still a call, so the check that
refuses a program in which nothing is a call still passes; every `fc-*`
birthmark simply comes out empty, and two empty birthmarks score as a perfect
match. It was written backwards first and did exactly that.

## ✍️ Replacing the script

`--script` takes an IDAPython file in place of the built-in one, run as

```
idat -A -c -o<database> -S"<your script> <maturity> <output name>" <binary>
```

with the working directory set to where the output belongs. Three obligations:

- **`json.dumps`, never string concatenation** (#77).
- **Sort anything iterated from a set.** The Binary Ninja script did not and
  wrote a different byte order on every run.
- **Key the symbol table the way your operations render a callee.**

## 🧪 Testing

IDA cannot be installed on a CI runner, so nothing in CI lifts — see [the note
on all the lifters](../README.md). The script is parsed on every push, and
fixtures under `testdata/lifted/mcode_*/` drive `tests/ida_test.rs`.

The test that matters most asserts that every maturity **resolves** the call in
the fixture, not merely that one is present. Both bugs found while writing this
were of that shape.
