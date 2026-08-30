# Supporting Binary Ninja and IDA Pro as lifters

Research note, August 2026. Everything stated about this repository was checked
against the code at `3bc95f6` and, where it concerns behaviour, reproduced by
running the tool. Claims about the Binary Ninja and IDA Pro APIs are marked as
needing confirmation against the licences and versions actually purchased.

A Japanese translation is kept alongside as `multi-lifter-support.ja.md`.

## Summary

The change does **not** stay inside `lift`. Four separate places hard-code
Ghidra's P-Code, and two of them fail silently rather than loudly, which is the
part that makes this more than a plumbing exercise.

Before any of it, there is a live bug to fix: **the `fc-*` birthmark family is
empty today**, and two unrelated binaries compare as identical through it.

Cross-tool comparison splits in two. The `op-*` family cannot work across
lifters and should be prevented rather than left to produce a misleading number.
The `fc-*` family plausibly can, because it holds symbol names taken from the
binary rather than opcodes taken from the IR.

## 1. A blocking bug, found while investigating

`extract_function_calls` (`src/extractor.rs:170`) looks the call target up in
the symbol table using the raw operand string:

```rust
f.iter()
    .filter(|op| op.mnemonic() == "CALL")
    .filter_map(|op| op.inputs().first().and_then(|addr| p.symbol(addr)))
```

`inputs().first()` is `"(ram, 0x100000480, 8)"`, while the `symbols` map is
keyed `"0x100000480"`. The lookup never matches, so `filter_map` discards every
call.

Reproduced on unmodified test data, current `main`:

```
$ oinkie extract -b fc-seq -d bm testdata/hello_world/pcodes/*.json
  hello_clang    [{'Seq': []}]
  hello_gcc      [{'Seq': []}]

$ oinkie compare -a jaccard -A hungarian -s all -d sim bm/*.json
  similarity: 1        # two different binaries, reported identical

$ oinkie run -a fc-set-jaccard -s all -d run testdata/hello_world/pcodes/*.json
  similarity: 1
```

Both empty sets, and `jaccard_index` returns `1.0` for two empty sets, so the
score is a **false positive**: the tool claims two unrelated programs match.
This affects all eight `fc-*` analyses offered by the CLI, through both
`compare` and `run`.

The fix is to parse the operand before the lookup — `ghidra::Value` already
does exactly that parsing (`src/ghidra.rs`), so the address can be formatted
back into the map's key form. Tracked as #40, and it should land before any
multi-lifter work: it is unrelated to it and currently produces wrong answers.

## 2. What is Ghidra-specific today

| # | Place | What it assumes | How it fails for another lifter |
|---|---|---|---|
| 1 | `LifterBuilder::build` (`src/lift.rs`) | only `Ghidra` is implemented | loud, already handled |
| 2 | `PcodeOp` (`src/ghidra/pcode.rs`) | a closed enum of 75 P-Code opcodes | loud — deserialization fails |
| 3 | `cli/main.rs:6,50,54,322` | `Program<oinkie::ghidra::Op>` | compile-time, needs a decision |
| 4 | `extract_function_calls` | the call opcode is named `"CALL"` | **silent** — empty birthmark |
| 5 | `Metadata` / `comparable_with` | no record of which lifter produced it | **silent** — nonsense comparison |

### 2 is loud, and that is fine

```
$ oinkie extract -b op-seq -d out bn.json
Error: bn.json: JSON error: unknown variant `LLIL_SET_REG`,
       expected one of `UNIMPLEMENTED`, `COPY`, `LOAD`, ...
```

A closed enum refusing a foreign vocabulary is the correct behaviour. Note
`Op::code()`, the only method that needs the numeric opcode, **has no callers**
— the birthmark is built entirely from `mnemonic()`, a string. The typed enum
is therefore earning very little at present.

### 4 is silent, and that is the dangerous one

`"CALL"` is P-Code's spelling. IDA microcode and Binary Ninja LLIL name the
operation differently, so the filter matches nothing and the birthmark comes
out empty — the same state as the bug in section 1, with the same false
positive. Simulated by renaming `CALL` to another valid P-Code opcode:

```
  hello_clang    elements=1  data=[{'Set': []}]
  hello_gcc      elements=1  data=[{'Set': []}]
  similarity: 1
```

### 5 is silent too

`Metadata` records `file_name`, `path`, `extracted_at`, `duration` and
`birthmark_type` — nothing about the lifter. `comparable_with` only checks
`birthmark_type`, so a Ghidra birthmark and an IDA birthmark of the same type
are considered comparable. Simulated by rewriting one birthmark's mnemonics
into a microcode-style vocabulary:

```
$ oinkie compare -a jaccard -A hungarian -s all -d sim ghidra.json ida_like.json
$ echo $?
0
  similarity: 0     # same binary, reported as completely unrelated
```

Exit code 0, no warning. A false negative to sit beside the false positive
above.

## 3. Can birthmarks be compared across lifters?

**`op-*`: no.** The three IRs disagree on both vocabulary and granularity. One
machine instruction becomes a different number of IR operations in each, so
even a perfect opcode mapping would not align the sequences, and the frequency
vectors would be scaled differently. The set-based variants would compare two
disjoint symbol sets and return ~0. This should be **rejected**, not computed.

**`fc-*`: plausibly yes** — and this is the part worth noticing. Despite the
name, `fc-*` birthmarks do not hold IR opcodes at all. `extract_function_calls`
maps a call target address to a **symbol name**, so the birthmark contains
entries like `_printf`, which come from the binary's symbol and import tables
rather than from the lifter's IR. Two tools reading the same binary should
recover substantially the same names.

Two obstacles, both tractable:

- each lifter names the call operation differently, which is issue 4 above
- symbol spelling varies by tool and platform (`_printf` vs `printf`), so a
  normalisation step is needed before comparison

That makes `fc-*` the realistic path to cross-tool comparison, and it is worth
saying so explicitly because the intuitive answer — "different IRs, so nothing
compares" — is wrong for this family.

## 4. Converting Ghidra output to IDA Pro's form

Two different questions, with different answers.

**Converting the IR is not practical.** P-Code to microcode is a translation
between two compiler intermediate representations with different structure and
granularity. It is a compiler project, and a lossy one: a single P-Code
operation may correspond to zero or several microcode operations depending on
context, so the result would not resemble what IDA itself produces. Nothing
would be gained over simply running IDA.

**Converting the birthmark is feasible for `fc-*` and lossy for `op-*`.** For
`fc-*` it reduces to symbol-name normalisation, which is a mapping table. For
`op-*` an opcode mapping could be written, but the granularity mismatch means
sequence-based and frequency-based variants would still be wrong; only the
set-based ones might be approximately meaningful, and even then the
normalisation would need validating experimentally before any result could be
trusted.

## 5. Design options

### A. String mnemonics, one shared `Op` type

Replace the `PcodeOp` enum with a plain `String` mnemonic. The birthmark only
uses `mnemonic()`, and `code()` has no callers, so almost nothing is lost.

- smallest change; removes hard-coding 2 and 3 at once
- gives up the validation that currently rejects a malformed P-Code file
- still needs 4 and 5 solved separately

### B. Per-lifter `Op` types, dispatched on a field in the JSON

Add a `lifter` field to the JSON, keep `PcodeOp` for Ghidra, add equivalents
for the others, and pick the type when loading.

- keeps per-IR validation
- the dispatch has to live somewhere that today just writes `Program<Op>`;
  `Program<T>` is generic, so the machinery is half there already
- the `lifter` field also supplies what section 5 needs for provenance

### C. Normalise every IR to a common vocabulary at lift time

- the only option that makes `op-*` comparable across tools
- requires a defensible semantic mapping, and its validity is a research
  question in itself rather than an implementation detail
- worth treating as a possible outcome of the work, not a prerequisite

**B is the recommendation.** It carries the provenance needed to make
comparisons safe, and A can still be adopted inside it later if the typed
opcodes prove not to earn their keep.

## 6. Suggested sequence

1. **Fix `fc-*`** (section 1). Independent, currently wrong, and it makes the
   family that has the best cross-tool prospects actually work.
2. **Record the lifter in the birthmark** and extend `comparable_with` to
   refuse a cross-lifter `op-*` comparison. This makes every later step fail
   loudly instead of silently, and is worth doing before any second lifter
   exists.
3. **Make the call opcode per-lifter** rather than the literal `"CALL"`.
4. **Add the `lifter` field to the JSON** and dispatch the `Op` type on it.
5. **Implement one new lifter end to end** — Binary Ninja first, see below.
6. **Then the second**, which should mostly be script work if 1–4 are right.
7. **Optionally**, investigate a normalised vocabulary (option C) for `op-*`.

Steps 1–4 are all inside this repository and are the bulk of the design work.
Steps 5–6 are mostly writing the equivalent of `HighPCodeLifter.java` (131
lines) for each tool.

## 7. Practical notes on the two tools

These need confirming against the licences and versions purchased.

**Binary Ninja.** Headless automation is a licence-tier feature; the Personal
tier has historically not permitted it, while Commercial and Ultimate do. The
Python API (`binaryninja`) is well documented and the BNIL family (LLIL, MLIL,
HLIL, and their SSA forms) gives a choice of abstraction level. **Which level
to lift from is a decision that affects birthmark quality and should be
settled experimentally, not assumed** — LLIL is closest to P-Code in spirit,
MLIL closer to what Ghidra's decompiler-based `HighFunction` currently gives.

**IDA Pro.** The microcode is part of the Hex-Rays decompiler, so a decompiler
licence is required in addition to IDA itself. Access is via `ida_hexrays` from
IDAPython, and headless runs use `idat -A -S<script>`. Note the existing Ghidra
script uses `DecompInterface` and `HighFunction`, so it already lifts from a
decompiler-level representation — microcode is the closer analogue, not the
disassembly.

Both integrate the same way Ghidra does: run the tool headless with a script
that writes the JSON, then move it into place. `GhidraLifter` is a reasonable
template, including the working-directory handling that #36 corrected.

## 8. Open questions

- Which BNIL level, and which microcode maturity level, produce birthmarks
  comparable in quality to the current Ghidra output? Experimental.
- Does the `symbols` map survive the change unaltered, or do the other tools
  need a different address format? Affects the fix in section 1.
- Should a cross-lifter `op-*` comparison be an error, or a warning with a
  computed score? An error is consistent with the decision taken in #38 to
  reject pairings that cannot mean what they say.
- Is `fc-*` cross-tool comparison accurate enough to be worth claiming? Needs
  measuring once section 1 is fixed and a second lifter exists.
