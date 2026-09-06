# Binary Lifter with Ghidra

This directory contains a script `lift.sh` that performs binary lifting using Ghidra. The script takes a target binary as an argument and generates a JSON file containing the lifted PCode representation of the binary.

## 🏃 Usage

To use the `lift.sh` script, run the following command in your terminal:

```sh
assets/lifters/ghidra/lift.sh /path/to/target/binary
```

The `lift.sh` assumes that:

- the working directory is the root of the project (the parent directory of `lifter`), and
- the path of binary files contains `executables`.

The script is to run Ghidra with headless mode (without GUI).

## the Generated JSON

### Examples

```json
{
    "program": "factorizer", // The name of the lifted program
    "path": "/home/tamada/products/oinkie/testdata/factorizer/factorizer.json", // The path to the generated JSON file
    "symbols": {       // A mapping of addresses to function names.
        "0x1000006b0": "_atoll",
        "0x1000006bc": "_printf",
        "0x100000460": "_factorize"
    },
    "functions": [
        {
            "name": "_factorize",
            "ops": [
                {"op": "CALL", "inputs": ["(ram, 0x1000006bc, 8)", "(unique, 0x1000006d, 8)"]},
                {"op": "COPY", "out": "(unique, 0x1000006d, 8)", "inputs": ["(const, 0x1000006c8, 8)"]},
                {"op": "INT_SLESS", "out": "(unique, 0x2200, 1)", "inputs": ["(register, 0x4000, 8)", "(const, 0x2, 8)"]},
                ...
            ]
        },
        {
            "name": "entry",
            "ops": [
                {"op": "INT_SLESS", "out": "(unique, 0x2200, 1)", "inputs": ["(register, 0x4000, 4)", "(const, 0x2, 4)"]},
                {"op": "CBRANCH", "inputs": ["(ram, 0x100000614, 1)", "(unique, 0x2200, 1)"]},
                ...
            ]
        }
    ]
}
```

## 🔁 Replacing the script

`oinkie lift --script <FILE>` replaces the built-in script with one of your own.
The language is the lifter's own — Java, for Ghidra — and the script runs
*inside* the tool, with everything the tool can reach. `--script` is not a
sandbox.

A replacement has to satisfy four things. Two of them are not guessable, and
one of them the built-in script itself got wrong.

### It must be named after its public class

Ghidra compiles the file and rejects `mylifter.java` holding
`public class HighPCodeLifter`:

```
mylifter.java:20: error: class HighPCodeLifter is public, should be declared in a file named HighPCodeLifter.java
```

You will not see that message from `oinkie`. `analyzeHeadless` exits 0 whether
or not its script compiled, so the lift is reported as having produced no
output, and the compiler's diagnostic is inside the captured process output.

### It must write `{input file name}.json` into its working directory

That is where `oinkie` looks for the result, and it is why the built-in script
writes to `Path.of(".")` rather than to a path of its own choosing.

### It must write valid JSON, escaping the names it embeds

Names come from the binary, not from you. A demangled C++ symbol can contain a
double quote — a user-defined literal operator becomes `operator""__km` — and
building JSON by concatenation then produces a file that cannot be parsed.

This is not hypothetical advice. The built-in script did exactly this until
v0.5.0, and the only way out of an affected file was to edit it by hand.

### A backslash is the dangerous one, not the quote

A `"` breaks the parse, which is loud, and you will find out immediately. A `\`
does not. A name written as `\t` is read back as a name holding a tab, the file
parses, and the birthmark is extracted from a name the program does not have —
so a wrong similarity score, with nothing to indicate it.

Escape `"` and `\`, the five characters with short forms, every other control
character below `0x20` as `\uXXXX`, and unpaired surrogates as `\uXXXX`. `/`
and non-ASCII are legal raw in a UTF-8 JSON file and need no escaping. See
`q()` in [`scripts/HighPCodeLifter.java`](scripts/HighPCodeLifter.java) for the
whole of it — it is about fifteen lines.

## ✅ Checking a script compiles

Ghidra compiles the script at lift time, so an error in it is not a build
failure — it is a lift that produces nothing, for the reason above. To find out
in about two seconds instead of a Ghidra round trip:

```sh
just compile-check                       # the built-in script
assets/lifters/ghidra/compile-check.sh path/to/Yours.java
```

Ghidra is found the way `oinkie` finds it: `GHIDRA_HOME`, then
`GHIDRA_INSTALL_DIR`, then the usual install locations.

The script needs a JDK at least as new as Ghidra's own class files (21 for
Ghidra 12). An older `javac` fails with `bad class file` against Ghidra's jars
and says nothing about your script at all, so the check refuses to run rather
than let you read that message; point `JAVA_HOME` at a newer JDK.

CI runs the same check on every push.
