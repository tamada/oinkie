"""Writes a function's Hex-Rays microcode to the Oinkie IR.

Run by `oinkie lift` as

    idat -A -c -S"MicrocodeLifter.py <maturity> <output-name>" <binary>

with the working directory set to where the JSON should land. `<maturity>` is
one of the names below and decides both which stage of the microcode is read
and which `ir` value the file carries.

Producing microcode runs the decompiler. On an installation whose only
decompiler is the cloud one, that sends each function to Hex-Rays' servers;
`oinkie lift` says so before starting this script.

Three things here are contracts rather than choices.

The output is built with `json.dumps` and never by concatenating strings. The
Ghidra script did the latter and produced files that could not be parsed once a
symbol held a quote, and files that parsed into the wrong name once one held a
backslash (#77).

The symbol table is keyed by the callee as the first operand renders it,
because that is what the Rust side looks up.

Everything iterated is sorted or written in the order IDA gives it, never out
of a set. The Binary Ninja script collected keys in a set and so wrote a
different byte order on every run.
"""

import json
import sys

import ida_auto
import ida_funcs
import ida_hexrays
import ida_pro
import idautils
import idc

# The maturities, and the constant each names. MMAT_ZERO is absent: it is the
# state before any microcode exists, so there is nothing there to write.
MATURITIES = {
    "generated": "MMAT_GENERATED",
    "preoptimized": "MMAT_PREOPTIMIZED",
    "locopt": "MMAT_LOCOPT",
    "calls": "MMAT_CALLS",
    "glbopt1": "MMAT_GLBOPT1",
    "glbopt2": "MMAT_GLBOPT2",
    "glbopt3": "MMAT_GLBOPT3",
    "lvars": "MMAT_LVARS",
}

# Opcode number to name, read from the module rather than written out. The
# microcode's vocabulary belongs to IDA and grows with releases; a list copied
# into this file would be a list to keep in step with one nobody controls.
OPCODE_NAMES = {
    value: name
    for name, value in vars(ida_hexrays).items()
    if name.startswith("m_") and isinstance(value, int)
}

# The operations that transfer control to another function. Both spellings
# count: m_call is direct, m_icall indirect. The Rust side holds the same pair,
# and the fixtures are what say the two agree.
CALL_OPS = ("m_call", "m_icall")


def operand(mop):
    """An operand as IDA renders it, or None when there is none."""
    if mop is None or mop.t == ida_hexrays.mop_z:
        return None
    return mop.dstr()


def walk(insn):
    """An instruction and every sub-instruction inside its operands.

    A microcode operand can be an instruction of its own. The optimiser folds a
    call into whatever consumes its result, so at MMAT_CALLS `_main`'s only call
    sits inside an `m_xds` operand rather than in the block's list. Walking only
    the list left that maturity with no call at all -- and `extract` refusing
    every fc-* birthmark of it, which is the refusal working but for the wrong
    reason.

    Sub-instructions come first, the way an operand is evaluated before the
    operation that reads it.
    """
    for mop in (insn.l, insn.r, insn.d):
        if mop is not None and mop.t == ida_hexrays.mop_d and mop.d is not None:
            for nested in walk(mop.d):
                yield nested
    yield insn


def instructions(mba):
    """Every instruction of every block, in the order IDA holds them."""
    for i in range(mba.qty):
        block = mba.get_mblock(i)
        insn = block.head
        while insn:
            for one in walk(insn):
                yield one
            insn = insn.next


def lift_function(ea, maturity):
    """The microcode of one function, or None when there is none to read."""
    func = ida_funcs.get_func(ea)
    if func is None:
        return None
    ranges = ida_hexrays.mba_ranges_t(func)
    failure = ida_hexrays.hexrays_failure_t()
    mba = ida_hexrays.gen_microcode(
        ranges, failure, None, ida_hexrays.DECOMP_NO_WAIT, maturity
    )
    if mba is None:
        # Reported rather than swallowed: a function the decompiler refused is
        # a gap in the birthmark, and a lift that hid it would look complete.
        sys.stderr.write(
            "oinkie: %s: no microcode (%s)\n" % (idc.get_func_name(ea), failure.str)
        )
        return None

    ops = []
    for insn in instructions(mba):
        entry = {"op": OPCODE_NAMES.get(insn.opcode, "m_unknown_%d" % insn.opcode)}
        inputs = [operand(insn.l), operand(insn.r)]
        entry["inputs"] = [i for i in inputs if i is not None]
        destination = operand(insn.d)
        if destination is not None:
            entry["out"] = destination
        ops.append(entry)
    return ops


def lift(maturity):
    functions = []
    symbol_keys = []
    for ea in idautils.Functions():
        ops = lift_function(ea, maturity)
        if ops is None:
            continue
        functions.append({"name": idc.get_func_name(ea), "ops": ops})
        for op in ops:
            if op["op"] in CALL_OPS and op["inputs"]:
                symbol_keys.append(op["inputs"][0])
    return functions, symbol_keys


def symbol_table(keys):
    """Each call-target key mapped to the name it stands for.

    The key is the operand exactly as the microcode rendered it, because that
    is what the Rust side looks up. The microcode writes a global as `$name`,
    so the `$` has to come off before the name can be resolved and has to stay
    on in the key. Getting that wrong is not loud: `m_call` is still a call, so
    the check that refuses a program in which nothing is a call still passes,
    and every fc-* birthmark comes out empty -- two of which score as a perfect
    match. This was written the other way first and did exactly that.

    A key that resolves to nothing -- a register holding an indirect target --
    is left out, which keeps a missing entry distinguishable from a wrong one.
    """
    table = {}
    for key in sorted(set(keys)):
        name = key[1:] if key.startswith("$") else key
        if idc.get_name_ea_simple(name) != idc.BADADDR:
            table[key] = name
    return table


def main(argv):
    if len(argv) != 3:
        sys.stderr.write("usage: MicrocodeLifter.py <maturity> <output-name>\n")
        return 2
    maturity_name, output = argv[1], argv[2]
    if maturity_name not in MATURITIES:
        sys.stderr.write(
            "unknown maturity %r, expected one of %s\n"
            % (maturity_name, ", ".join(sorted(MATURITIES)))
        )
        return 2

    ida_auto.auto_wait()
    if not ida_hexrays.init_hexrays_plugin():
        sys.stderr.write("oinkie: the Hex-Rays decompiler is not available\n")
        return 3
    maturity = getattr(ida_hexrays, MATURITIES[maturity_name])

    functions, keys = lift(maturity)
    document = {
        "program": idc.get_root_filename(),
        "path": idc.get_input_file_path(),
        "ir": "ida-microcode-%s" % maturity_name,
        "symbols": symbol_table(keys),
        "functions": functions,
    }
    with open(output, "w", encoding="utf-8") as out:
        json.dump(document, out, ensure_ascii=False, indent=2)
        out.write("\n")
    return 0


if __name__ == "__main__":
    ida_pro.qexit(main(idc.ARGV))
