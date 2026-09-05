import ghidra.app.script.GhidraScript;

import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Writes an unreadable file and reports success, on purpose.
 *
 * This is the shape every lifting script can fail in and none can detect for
 * itself: the script writes bytes, returns normally, and analyzeHeadless exits
 * 0 whether or not anything downstream can read what it wrote. Before #83, a
 * lift like this one succeeded, and the failure surfaced at `extract` as a
 * parse error naming a line and column in a file the reader had never seen.
 *
 * Used by tests/cli_test.rs through `--script`, which is also the point: a
 * replacement script is arbitrary Java that oinkie never inspects, so the
 * check has to be on the output rather than on the script.
 */
public class BrokenLifter extends GhidraScript {

    @Override
    public void run() throws Exception {
        Path cwd = Path.of(".");
        // Valid JSON as far as the first two fields, then truncated -- the way
        // a script that dies part-way through writing leaves things.
        Files.writeString(
            cwd.resolve(currentProgram.getName() + ".json"),
            "{\"program\": \"broken\", \"ir\": \"ghidra-pcode\", \"functions\": [");
    }
}
