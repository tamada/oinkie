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
        jsonOutput.add(String.format("  \"program\": %s,", q(currentProgram.getName())));
        jsonOutput.add(String.format("  \"path\": %s,", q(path.toString())));
        // Names the intermediate representation, not the tool: one tool can
        // produce several and they are not interchangeable. This is decompiler
        // P-Code, from HighFunction, rather than raw lifted P-Code.
        jsonOutput.add("  \"ir\": \"ghidra-pcode\",");

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
            .map(e -> String.format("    %s: %s", q("0x" + e.getKey()), q(e.getValue())))
            .collect(Collectors.joining(",\n"));
        jsonOutput.add(items);
        jsonOutput.add("  },");
        // 関数ブロックをカンマで結合して追加
        jsonOutput.add("  \"functions\": [");
        jsonOutput.add(String.join(",\n", functionBlocks));
        jsonOutput.add("  ]");
        jsonOutput.add("}");

        outputToFile(currentProgram.getName(), jsonOutput);

        decompInterface.dispose();
    }

    private void outputToFile(String fileName, List<String> outputs) throws IOException {
        Path cwd = Path.of(".");
        try (var out = Files.newBufferedWriter(cwd.resolve(fileName + ".json"))) {
            var w = new java.io.PrintWriter(out);
            outputs.stream()
                .forEach(line -> w.println(line));
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
            "    {\n      \"name\": %s,\n      \"ops\": [\n%s\n      ]\n    }",
            q(func.getName()),
            opsJson.stream().map(s -> "        " + s).collect(Collectors.joining(",\n"))
        );
    }

    private void pushSymbolsIfNeeded(PcodeOpAST op, HashMap<String, String> symbols) {
        if (op.getOpcode() == PcodeOp.CALL) {
            Varnode target = op.getInput(0);
            if (target != null && target.isAddress()) {
                Address addr = target.getAddress();
                // GhidraのAPIでその場所にある関数を取得
                Function targetFunc = getFunctionAt(addr);
                if (targetFunc != null) {
                    // symbolsマップに "0x401234": "func_name" の形式で保存
                    symbols.put(addr.toString(), targetFunc.getName());
                }
            }
        }
    }

    private String getOpJson(PcodeOp op) {
        String mnemonic = op.getMnemonic();
        Varnode out = op.getOutput();
        Varnode[] inputs = op.getInputs();

        // 入力Varnodeのリストを作成
        List<String> inputStrings = new ArrayList<>();
        for (Varnode in : inputs) {
            inputStrings.add(q(in.toString()));
        }
        String inputsJson = String.join(", ", inputStrings);

        // 出力Varnodeの有無でフォーマットを分ける
        if (out != null) {
            return String.format(
                "{\"op\": %s, \"out\": %s, \"inputs\": [%s]}",
                q(mnemonic), q(out.toString()), inputsJson
            );
        } else {
            return String.format(
                "{\"op\": %s, \"inputs\": [%s]}",
                q(mnemonic), inputsJson
            );
        }
    }

    /**
     * Returns str as a JSON string literal, quotes included.
     *
     * Everything JSON forbids raw inside a string is escaped: the two structural
     * characters, the five with short forms, and every other control character
     * below 0x20. Unpaired surrogates are escaped too -- not a JSON matter but an
     * encoding one, since the writer encodes UTF-8 and would otherwise throw
     * part-way through the file.
     */
    private static String q(String str) {
        StringBuilder sb = new StringBuilder(str.length() + 16);
        sb.append('"');
        for (int i = 0; i < str.length(); i++) {
            char c = str.charAt(i);
            switch (c) {
                case '"':  sb.append("\\\""); break;
                case '\\': sb.append("\\\\"); break;
                case '\b': sb.append("\\b");  break;
                case '\f': sb.append("\\f");  break;
                case '\n': sb.append("\\n");  break;
                case '\r': sb.append("\\r");  break;
                case '\t': sb.append("\\t");  break;
                default:
                    if (c < 0x20 || isUnpaired(str, i)) {
                        sb.append(String.format("\\u%04x", (int) c));
                    } else {
                        sb.append(c);
                    }
                    break;
            }
        }
        return sb.append('"').toString();
    }

    private static boolean isUnpaired(String s, int i) {
        char c = s.charAt(i);
        if (Character.isHighSurrogate(c)) {
            return i + 1 >= s.length() || !Character.isLowSurrogate(s.charAt(i + 1));
        }
        if (Character.isLowSurrogate(c)) {
            return i == 0 || !Character.isHighSurrogate(s.charAt(i - 1));
        }
        return false;
    }
}
