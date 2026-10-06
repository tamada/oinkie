import com.google.gson.JsonParser;

import java.lang.reflect.Method;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

/**
 * Tests the string handling in HighPCodeLifter.java without running Ghidra.
 *
 * Its two pure static methods are where its logic can go wrong without a
 * Program: `q`, the JSON escaper, and `programPath`, Ghidra's spelling of a
 * Windows path. Both are private, so they are reached by reflection, and the rest
 * of the script -- which needs a Program and a decompiler -- is left to a
 * real lift.
 *
 * No JUnit: this is compiled beside the script by compile-check.sh --run,
 * against Ghidra's jars and nothing else, and exits non-zero on a failure.
 */
public class HighPCodeLifterTest {

    private static final List<String> failures = new ArrayList<>();

    public static void main(String[] args) throws Exception {
        Method q = HighPCodeLifter.class.getDeclaredMethod("q", String.class);
        q.setAccessible(true);
        Method programPath = HighPCodeLifter.class.getDeclaredMethod("programPath", String.class);
        programPath.setAccessible(true);

        // Each one is read back with a JSON reader rather than compared to the
        // bytes expected, so what is asserted is "valid JSON meaning this
        // string" -- which is what extract needs -- and not one spelling of it.
        String[][] strings = {
            {"plain", "main"},
            {"empty", ""},
            {"a quote", "operator\"\"_km"},
            {"a backslash", "C:\\path\\name"},
            {"the five short forms", "\b\f\n\r\t"},
            {"a control character without a short form", "a\u0001b\u001fc"},
            {"a lone high surrogate", "x\ud800y"},
            {"a high surrogate at the end", "x\ud800"},
            {"a lone low surrogate", "\udc00y"},
            {"a pair the wrong way round", "\udc00\ud800"},
            {"a pair", "emoji \ud83d\ude00"},
        };
        for (String[] c : strings) {
            String name = c[0];
            String s = c[1];
            String json = (String) q.invoke(null, s);
            checkQ(name, s, json);
        }
        // A correctly paired surrogate is a character, and is written as one.
        String pair = (String) q.invoke(null, "\ud83d\ude00");
        expect("a pair is not escaped", pair.equals("\"\ud83d\ude00\""), pair);

        String[][] paths = {
            {"a Unix path", "/usr/bin/ls", "/usr/bin/ls"},
            {"Ghidra's Windows path", "/D:/a/oinkie/hello.exe", "D:/a/oinkie/hello.exe"},
            {"a lower-case drive", "/c:/x.exe", "c:/x.exe"},
            {"a Unix path whose third character is not a colon", "/ab/c", "/ab/c"},
            {"too short to hold a drive", "/D", "/D"},
        };
        for (String[] c : paths) {
            checkPath(programPath, c[0], c[1], c[2]);
        }
        // A colon in third place is not a drive unless a letter precedes it,
        // and this one is a Unix path that has to be left alone. Not asked on
        // Windows, where no Path can spell it.
        if (java.io.File.separatorChar == '/') {
            checkPath(programPath, "a digit before the colon", "/1:/x", "/1:/x");
        }

        if (!failures.isEmpty()) {
            failures.forEach(f -> System.err.println("FAIL: " + f));
            System.exit(1);
        }
        System.out.println("ok: HighPCodeLifter's q and programPath");
    }

    private static void checkPath(Method programPath, String name, String given, String expected)
            throws Exception {
        Path got = (Path) programPath.invoke(null, given);
        Path want = Path.of(expected);
        expect(name, got.equals(want), given + " -> " + got + ", not " + want);
    }

    private static void checkQ(String name, String s, String json) {
        String read;
        try {
            read = JsonParser.parseString(json).getAsString();
        } catch (RuntimeException e) {
            failures.add(name + ": " + json + " does not parse: " + e.getMessage());
            return;
        }
        expect(name + ": reads back as itself", read.equals(s), json);
        // Gson reads leniently, so what it would let through is checked too:
        // nothing JSON forbids raw, and nothing UTF-8 cannot encode.
        expect(name + ": no raw control character",
            json.chars().noneMatch(ch -> ch < 0x20), json);
        String roundTrip = new String(json.getBytes(StandardCharsets.UTF_8), StandardCharsets.UTF_8);
        expect(name + ": encodes as UTF-8", roundTrip.equals(json), json);
    }

    private static void expect(String what, boolean ok, String detail) {
        if (!ok) {
            failures.add(what + ": " + escapeForReport(detail));
        }
    }

    /** The detail, readable in a log whatever it holds. */
    private static String escapeForReport(String s) {
        StringBuilder sb = new StringBuilder();
        for (char c : s.toCharArray()) {
            if (c < 0x20 || c >= 0x7f) {
                sb.append(String.format("\\u%04x", (int) c));
            } else {
                sb.append(c);
            }
        }
        return sb.toString();
    }
}
