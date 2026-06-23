---
title: "1. Lifting Binaries (`lift` command)"
description: "How to lift binary executable files to P-code JSON format using Ghidra."
date: 2026-06-22
draft: false
weight: 50
---

The **lift** command converts compiled binary executable files into the Oinkie Intermediate Representation (OIR) JSON format. The current implementation utilizes **Ghidra** in headless mode (without GUI) to translate machine-specific assembly instructions into platform-agnostic P-code.

---

## 🏃 Usage

You can lift multiple binary files at once. The default behavior is to use Ghidra to analyze the executables and output the resulting JSON files to the `./pcodes` directory.

```sh
oinkie lift [OPTIONS] [FILES]...
```

### Arguments
* `<FILES>...`  
  Path to the binary or intermediate files to lift.

### Options
* `-d, --dest <DIRECTORY>`  
  Specify the directory to place the resulting JSON files of the lifted P-code. Defaults to the `./pcodes` directory. `[default: pcodes]`
* `-l, --lifter-type <LIFTER_TYPE>`  
  Specify the lifter type to use. `[default: ghidra]` `[possible values: ghidra, llvm, binary-ninja]`
* `-H, --home <HOME>`  
  Specify the path to the home directory of the lifter (e.g., `GHIDRA_HOME` for Ghidra). If not specified, the application will search the respective environment variable or look for common default paths.
* `-i, --intermediate <DIRECTORY>`  
  Specify a directory to keep intermediate lifter files (like Ghidra project directories). If not provided, a temporary directory is used and deleted automatically.
* `--script <SCRIPT>`  
  Path to a custom lifting script. The script's interpretation depends on the lifter type. For Ghidra, it's the path to a Java script.
* `-S, --skip`  
  Skip the lifting process if the output JSON file already exists in the destination directory.

---

## 🔍 Under the Hood: Ghidra Lifter

In the backend, `oinkie` runs a headless Ghidra process to analyze the binary structures, extract assembly symbols, and decompile or translate instructions into **P-code** (Ghidra's register-transfer language).

By default, the process automatically manages the creation, analysis, and clean-up of temporary Ghidra projects so that you only receive the final platform-independent OIR JSON representations.

---

## ☕ Default Ghidra Script: `HighPCodeLifter.java`

When using the default Ghidra lifter, **oinkie** deploys a specialized Ghidra Script to interface with the Ghidra Decompiler API. 

If you want to customize your lifting script (using the `--script` parameter), you can use the following Java source code of `HighPCodeLifter.java` as a foundational template. It decompiles each function, tracks `CALL` targets to populate resolved symbols, and formats instructions as P-code operations:

```java
import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.address.Address;
import ghidra.program.model.pcode.PcodeOp;
import ghidra.program.model.pcode.PcodeOpAST;
import ghidra.program.model.pcode.Varnode;
import ghidra.program.model.pcode.HighFunction;
import ghidra.program.model.listing.Function;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.HashMap;
import java.util.stream.Collectors;

public class HighPCodeLifter extends GhidraScript {

    @Override
    public void run() throws Exception {
        DecompInterface decompInterface = new DecompInterface();
        decompInterface.openProgram(currentProgram);
        var path = java.nio.file.Path.of(currentProgram.getExecutablePath());

        List<String> jsonOutput = new ArrayList<>();
        jsonOutput.add("{");
        jsonOutput.add(String.format("  \"program\": \"%s\",", currentProgram.getName()));
        jsonOutput.add(String.format("  \"path\": \"%s\",", path));

        List<String> functionBlocks = new ArrayList<>();
        Function func = getFirstFunction();
        HashMap<String, String> symbols = new HashMap<>();

        while (func != null && !monitor.isCancelled()) {
            if (!func.isThunk() && !func.isExternal()) {
                DecompileResults results = decompInterface.decompileFunction(func, 30, monitor);
                if (results != null && results.decompileCompleted()) {
                    functionBlocks.add(getFunctionJson(func, results.getHighFunction(), symbols));
                }
            }
            func = getFunctionAfter(func);
        }

        jsonOutput.add("  \"symbols\": {");
        String items = symbols.entrySet().stream()
            .map(e -> String.format("    \"0x%s\": \"%s\"", e.getKey(), e.getValue()))
            .collect(Collectors.joining(",\n"));
        jsonOutput.add(items);
        jsonOutput.add("  },");
        jsonOutput.add("  \"functions\": [");
        jsonOutput.add(String.join(",\n", functionBlocks));
        jsonOutput.add("  ]");
        jsonOutput.add("}");

        outputToFile(currentProgram.getName(), jsonOutput);

        decompInterface.dispose();
    }

    private void outputToFile(String fileName, List<String> outputs) {
        Path cwd = Path.of(".");
        try (var out = Files.newBufferedWriter(cwd.resolve(fileName + ".json"))) {
            var w = new java.io.PrintWriter(out);
            outputs.stream()
                .forEach(line -> w.println(line));
        } catch(java.io.IOException e) {
            e.printStackTrace();
        }
    }

    private String getFunctionJson(Function func, HighFunction highFunc, HashMap<String, String> symbols) {
        List<String> opsJson = new ArrayList<>();
        Iterator<PcodeOpAST> opIter = highFunc.getPcodeOps();

        while (opIter.hasNext()) {
            PcodeOpAST op = opIter.next();
            opsJson.add(getOpJson(op));
            pushSymbolsIfNeeded(op, symbols);
        }

        return String.format(
            "    {\n      \"name\": \"%s\",\n      \"ops\": [\n%s\n      ]\n    }",
            func.getName(),
            opsJson.stream().map(s -> "        " + s).collect(Collectors.joining(",\n"))
        );
    }

    private void pushSymbolsIfNeeded(PcodeOpAST op, HashMap<String, String> symbols) {
        if (op.getOpcode() == PcodeOp.CALL) {
            Varnode target = op.getInput(0);
            if (target != null && target.isAddress()) {
                Address addr = target.getAddress();
                Function targetFunc = getFunctionAt(addr);
                if (targetFunc != null) {
                    symbols.put(addr.toString(), targetFunc.getName());
                }
            }
        }        
    }

    private String getOpJson(PcodeOp op) {
        String mnemonic = op.getMnemonic();
        Varnode out = op.getOutput();
        Varnode[] inputs = op.getInputs();

        List<String> inputStrings = new ArrayList<>();
        for (Varnode in : inputs) {
            inputStrings.add(String.format("\"%s\"", in.toString()));
        }
        String inputsJson = String.join(", ", inputStrings);

        if (out != null) {
            return String.format(
                "{\"op\": \"%s\", \"out\": \"%s\", \"inputs\": [%s]}",
                mnemonic, out.toString(), inputsJson
            );
        } else {
            return String.format(
                "{\"op\": \"%s\", \"inputs\": [%s]}",
                mnemonic, inputsJson
            );
        }
    }
}
```

---

## 📄 The OIR (Oinkie IR) JSON Schema

The output is structured as a JSON file representing the program's functions and their corresponding P-code operations. 

Here is an example of the generated JSON file:

```json
{
    "program": "factorizer",
    "path": "/home/tamada/products/oinkie/testdata/factorizer/factorizer.json",
    "symbols": {
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
                {"op": "INT_SLESS", "out": "(unique, 0x2200, 1)", "inputs": ["(register, 0x4000, 8)", "(const, 0x2, 8)"]}
            ]
        },
        {
            "name": "entry",
            "ops": [
                {"op": "INT_SLESS", "out": "(unique, 0x2200, 1)", "inputs": ["(register, 0x4000, 4)", "(const, 0x2, 4)"]},
                {"op": "CBRANCH", "inputs": ["(ram, 0x100000614, 1)", "(unique, 0x2200, 1)"]}
            ]
        }
    ]
}
```

### JSON Property Descriptions
* **`program`**: The name of the lifted program.
* **`path`**: The absolute path to the generated JSON file.
* **`symbols`**: A mapping of instruction memory addresses to resolved function/external API names.
* **`functions`**: An array of objects representing each identified function in the binary.
  * **`name`**: The name of the function.
  * **`ops`**: A list of sequential P-code operations inside this function. Each operation contains:
    * `op`: The P-code operation type (e.g., `CALL`, `COPY`, `INT_SLESS`, `CBRANCH`).
    * `out`: (Optional) The output variable storage details.
    * `inputs`: (Optional) An array of inputs used by the instruction.
