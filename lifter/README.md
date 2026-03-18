# Binary Lifter with Ghidra

This directory contains a script `lift.sh` that performs binary lifting using Ghidra. The script takes a target binary as an argument and generates a JSON file containing the lifted PCode representation of the binary.

## 🏃 Usage

To use the `lift.sh` script, run the following command in your terminal:

```sh
lifter/lift.sh /path/to/target/binary
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
