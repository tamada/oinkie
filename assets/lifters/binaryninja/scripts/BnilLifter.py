"""Writes a Binary Ninja function's IL to the Oinkie IR.

Run by `oinkie lift` as

    bnpython3 BnilLifter.py <binary> <level> <output-name>

with the working directory set to where the JSON should land. `<level>` is
one of llil, mlil or hlil and decides both which IL is read and which `ir`
value the file carries.

Two things about this file are contracts rather than choices.

The output is built with `json.dumps` and never by concatenating strings. The
Ghidra script did the latter and produced files that could not be parsed once
a symbol held a quote, and files that parsed into the wrong name once one held
a backslash (#77). A second lifter in a second language is a second chance to
make that mistake.

The symbol table is keyed by whatever this level renders a call target as,
because that is what the Rust side will look up. LLIL and MLIL render it as a
hexadecimal address; HLIL has already resolved it to a name, so at that level
the key is the name. The Oinkie IR does not say the key must be an address --
it says the lifter that produced both sides reconciles them.
"""

import json
import os
import sys

import binaryninja as bn

LEVELS = ("llil", "mlil", "hlil")

# The operations that transfer control to another function, per level. These
# have to match what Binary Ninja actually emits: an `is_call` matching
# nothing leaves the fc-* birthmarks empty, and two empty birthmarks score as
# a perfect match, so unrelated programs would be reported as identical.
CALL_OPS = {
    "llil": ("LLIL_CALL", "LLIL_CALL_STACK_ADJUST", "LLIL_TAILCALL", "LLIL_SYSCALL"),
    "mlil": ("MLIL_CALL", "MLIL_CALL_UNTYPED", "MLIL_TAILCALL", "MLIL_TAILCALL_UNTYPED",
             "MLIL_SYSCALL", "MLIL_SYSCALL_UNTYPED"),
    "hlil": ("HLIL_CALL", "HLIL_TAILCALL", "HLIL_SYSCALL"),
}

# Which rendered operand names the callee, per level. LLIL puts the target
# first; MLIL puts its output variables first and the target second; HLIL
# renders the callee itself first.
CALL_TARGET_INDEX = {"llil": 0, "mlil": 1, "hlil": 0}


def il_for(func, level):
    """The requested IL, and only the requested one.

    Written as a chain rather than as a dict keyed by level, because a dict
    literal evaluates every value before it is indexed: `{"llil": func.llil,
    ...}[level]` asks Binary Ninja for all three ILs of every function in order
    to return one of them. That is analysis nobody asked for, and it makes a
    level that happens to be unavailable for one function fail a lift that
    never wanted it.
    """
    if level == "llil":
        return func.llil
    if level == "mlil":
        return func.mlil
    return func.hlil


def call_target(op, inputs, level):
    """The callee as this level renders it, or None when there is no operand."""
    if op not in CALL_OPS[level]:
        return None
    index = CALL_TARGET_INDEX[level]
    if index >= len(inputs):
        return None
    return inputs[index]


def lift(bv, level):
    functions = []
    symbol_keys = set()

    for func in bv.functions:
        il = il_for(func, level)
        if il is None:
            continue
        ops = []
        for instr in il.instructions:
            op = instr.operation.name
            inputs = [str(operand) for operand in instr.operands]
            entry = {"op": op, "inputs": inputs}
            written = [str(v) for v in getattr(instr, "vars_written", []) or []]
            if written:
                entry["out"] = written[0]
            ops.append(entry)
            target = call_target(op, inputs, level)
            if target is not None:
                symbol_keys.add(target)
        functions.append({"name": func.name, "ops": ops})

    return functions, symbol_keys


def symbol_table(bv, keys, level):
    """Maps each call-target key seen above to the name it stands for.

    At HLIL the key is already the name and the entry is an identity, which is
    the honest record of a level that resolved the symbol before oinkie saw it.

    The keys are sorted, and that is not cosmetic. They are collected in a set,
    whose iteration order varies between runs with Python's hash seed, so
    without this two lifts of one binary produce files that differ only in the
    order of this table -- 2794 differing lines on a Go binary, with every
    entry the same. Ghidra's lifter is byte-stable, a re-lift that changes
    nothing should show as changing nothing, and a fixture should be
    regenerable and diffable.
    """
    if level == "hlil":
        return {key: key for key in sorted(keys)}

    table = {}
    for key in sorted(keys):
        try:
            address = int(key, 16)
        except ValueError:
            # An indirect call through a register or a variable. There is no
            # name to find, and omitting it is what keeps a missing entry
            # distinguishable from a wrong one.
            continue
        symbol = bv.get_symbol_at(address)
        if symbol is not None:
            table[key] = symbol.name
    return table


def main(argv):
    if len(argv) != 4:
        sys.stderr.write("usage: BnilLifter.py <binary> <level> <output-name>\n")
        return 2
    binary, level, output = argv[1], argv[2], argv[3]
    if level not in LEVELS:
        sys.stderr.write("unknown level %r, expected one of %s\n" % (level, ", ".join(LEVELS)))
        return 2
    # oinkie names the output `{input file name}.json` and collects it from
    # the working directory it runs this in, so a name with a directory in it
    # is not one oinkie gave. Refused rather than written wherever it points
    # (SonarQube pythonsecurity:S8707, #169).
    name = os.path.basename(output)
    if name != output or name in ("", ".", ".."):
        sys.stderr.write("output %r must be a file name, without a directory\n" % output)
        return 2

    with bn.load(binary) as bv:
        functions, keys = lift(bv, level)
        document = {
            "program": bv.file.original_filename.rsplit("/", 1)[-1],
            "path": binary,
            "ir": "binary-ninja-%s" % level,
            "symbols": symbol_table(bv, keys, level),
            "functions": functions,
        }

    with open(os.path.join(os.getcwd(), name), "w", encoding="utf-8") as out:
        json.dump(document, out, ensure_ascii=False, indent=2)
        out.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
