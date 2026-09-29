# 🔬 The IDA Pro lifter

`oinkie lift` drives `idat` headless and reads back the Hex-Rays microcode.

```sh
oinkie lift -r ida-microcode path/to/binary
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
non-interactive run, and asking for `-r ida-microcode` is the consent, since
IDA has no other representation to ask for.

The warning is conditional, and this is how it decides. A decompiler plugin is
named `hex<arch>` when it decompiles locally and `hexc<arch>` when it calls
out. IDA Classroom ships the cloud ones; a Professional licence with the
decompiler ships local ones.

**A cloud plugin anywhere earns the warning, even beside local ones.** Each
plugin covers its own architectures, so `hexcx64` next to `hexarm64`
decompiles x64 in the cloud whatever the ARM64 plugin does. Deciding otherwise
would need the binary's architecture, which is not known until IDA has read it
— after the point where a warning is still worth giving. So the message says
what is installed rather than what will happen, and names both halves. An
over-warning costs attention; a missed one costs a function that has already
left the machine.

It goes straight to stderr rather than through the logger, so `--level error`
and `--level off` do not silence it. A privacy disclosure a verbosity flag can
turn off is one the user never agreed to turn off.

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

## 🧬 One representation, not eight

The decompiler rewrites the microcode through a pipeline, and `gen_microcode`
takes the stage to stop at. oinkie reads the last of them, `MMAT_LVARS`: local
variables allocated, and what the pseudocode is rendered from.

The earlier stages are not offered, and the reason is worth stating because
Binary Ninja's three levels look like a parallel and are not. LLIL, MLIL and
HLIL are representations Binary Ninja publishes, and picking one is a choice
its users are expected to make. The maturities are where a plugin may intervene
in the decompiler's own pipeline. The enum says so: `MMAT_ZERO` is in it,
described as "microcode does not exist", `MMAT_CALLS` is documented by pointing
at an event hook, and `MMAT_GLBOPT2` only as "most global optimization passes
are done".

Nothing states that the intermediate shapes are stable across releases, and a
representation named here is permanent — a file carrying the name has to stay
readable. Measured on one `hello world`, four of the eight produced the same
operations anyway: Hex-Rays says the microcode is fixed at `MMAT_GLBOPT3`, so
`MMAT_LVARS` differs from it only in what the variables are called.

Adding a maturity later costs nothing. Removing one breaks every file that
named it.

One thing the pipeline does that the reader has to account for: the optimiser
folds a call into whatever consumes its result, so a call can sit inside
another instruction's operand rather than in a block's own list. The script
walks into operands for that reason.

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
idat -A -c -o<database> -S"<your script> <output name>" <binary>
```

with the working directory set to where the output belongs. Three obligations:

- **`json.dumps`, never string concatenation** (#77).
- **Sort anything iterated from a set.** The Binary Ninja script did not and
  wrote a different byte order on every run.
- **Key the symbol table the way your operations render a callee.**

## 🚫 A refused function fails the lift

`gen_microcode` can decline a function. The script stops rather than skipping
it, and exits non-zero.

Continuing would be worse than it looks. `Headless` treats a process that exits
0 with an output file as a success and does not read its stderr, so a warning
would reach nobody — and the file would hold a program with functions missing
and no record that any were. Every birthmark taken from it would be computed
from a part of the program while claiming to describe the whole.

## 🧪 Testing

IDA cannot be installed on a CI runner, so nothing in CI lifts — see [the note
on all the lifters](../README.md). The script is parsed on every push, and
fixtures under `testdata/lifted/mcode/` drive `tests/ida_test.rs`.

The test that matters most asserts that the call in the fixture is **resolved**,
not merely that one is present. Both bugs found while writing this were of that
shape.
