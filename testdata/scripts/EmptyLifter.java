import ghidra.app.script.GhidraScript;

import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Writes a readable file with no functions in it, and reports success.
 *
 * The quieter sibling of BrokenLifter. That one writes bytes nothing can
 * parse, which at least announces itself; this one writes a file oinkie reads
 * without complaint and that says nothing, so every birthmark taken from it is
 * empty -- and two empty birthmarks score 1.0, which reports unrelated
 * programs as identical.
 *
 * Not a hypothetical shape. A Ghidra whose decompiler native binary is missing
 * produces exactly this: `DecompInterface` fails for every function,
 * HighPCodeLifter skips each one, and analyzeHeadless exits 0 (#54, #126).
 * That went unnoticed on two platforms until a test that wanted a particular
 * function name failed for want of any.
 *
 * Used by tests/cli_test.rs through `--script`, because the refusal has to be
 * on the output: a replacement script is arbitrary Java that oinkie never
 * inspects.
 */
public class EmptyLifter extends GhidraScript {

    @Override
    public void run() throws Exception {
        Path cwd = Path.of(".");
        Files.writeString(
            cwd.resolve(currentProgram.getName() + ".json"),
            "{\"program\": \"empty\", \"path\": \"bin/empty\", \"ir\": \"ghidra-pcode\","
                + " \"symbols\": {}, \"functions\": []}");
    }
}
