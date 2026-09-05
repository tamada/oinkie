#! /bin/bash
#
# Compiles the Ghidra lifting script, without running Ghidra.
#
# Ghidra compiles the script itself, at lift time, so a syntax or a type error
# in it is not a build failure here. It is a lift that produces nothing --
# reported as a missing output rather than as an error, because analyzeHeadless
# exits 0 whether or not its script compiled, and the compiler's diagnostic is
# buried in the process output. This catches it in about two seconds instead of
# a Ghidra round trip.
#
# Usage: assets/lifters/ghidra/compile-check.sh [script.java ...]
#
# Ghidra is found the way oinkie finds it: GHIDRA_HOME, then
# GHIDRA_INSTALL_DIR (which the CI action exports), then the usual install
# locations.

set -euo pipefail

# The lowest JDK that can read Ghidra's own class files. Ghidra 12 ships class
# file version 65, which is Java 21; an older javac rejects the jars with "bad
# class file" and says nothing about the script, which sends the reader looking
# in entirely the wrong place.
readonly MIN_JDK=21

scripts=("$@")
if [ ${#scripts[@]} -eq 0 ]; then
    scripts=("assets/lifters/ghidra/scripts/HighPCodeLifter.java")
fi

find_ghidra_home() {
    if [ -n "${GHIDRA_HOME:-}" ]; then
        echo "$GHIDRA_HOME"
        return
    fi
    if [ -n "${GHIDRA_INSTALL_DIR:-}" ]; then
        echo "$GHIDRA_INSTALL_DIR"
        return
    fi
    local candidate
    for candidate in /opt/homebrew/opt/ghidra/libexec \
                     /usr/local/opt/ghidra/libexec \
                     /opt/ghidra/libexec; do
        if [ -d "$candidate/Ghidra" ]; then
            echo "$candidate"
            return
        fi
    done
    echo "$0: cannot find Ghidra. Set GHIDRA_HOME to its installation directory." >&2
    exit 1
}

find_javac() {
    local javac
    if [ -n "${JAVA_HOME:-}" ] && [ -x "$JAVA_HOME/bin/javac" ]; then
        javac="$JAVA_HOME/bin/javac"
    elif javac=$(command -v javac); then
        :
    else
        echo "$0: no javac on PATH, and JAVA_HOME is unset or has none." >&2
        exit 1
    fi

    # "javac 25.0.2" -> 25, "javac 25-ea" -> 25, "javac 1.8.0_292" -> 1.
    #
    # Leading digits rather than the first dot-separated field, because an
    # early-access build reports "25-ea" and has no dot at all. Cutting on the
    # dot left that whole string, and `[ 25-ea -lt 21 ]` is not a comparison
    # that fails -- it is an error, which `if` reads as false, so the guard
    # waved the JDK through and printed a shell error next to it.
    local reported major
    reported=$("$javac" -version 2>&1 | awk '{ print $2 }')
    major=${reported%%[!0-9]*}
    if [ -z "$major" ]; then
        echo "$0: cannot tell which Java $javac is: it reports \"$reported\"." >&2
        echo "  Refusing rather than guessing, since the guard below is the" >&2
        echo "  only thing standing between you and an unreadable error." >&2
        exit 1
    fi
    if [ "$major" -lt "$MIN_JDK" ]; then
        echo "$0: $javac is Java $reported; Ghidra's class files need $MIN_JDK or newer." >&2
        echo "  An older javac fails with \"bad class file\" against Ghidra's jars," >&2
        echo "  which is about the JDK and not about the script." >&2
        echo "  Point JAVA_HOME at a newer JDK." >&2
        exit 1
    fi
    echo "$javac"
}

ghidra_home=$(find_ghidra_home)
javac=$(find_javac)

if [ ! -d "$ghidra_home/Ghidra" ]; then
    echo "$0: $ghidra_home does not look like a Ghidra installation (no Ghidra/ inside)." >&2
    exit 1
fi

# The trailing separator is stripped: Java reads an empty classpath element as
# the current directory, so leaving it there would put "." on the classpath and
# let a stray .class file stand in for a jar that is actually missing.
classpath=$(find "$ghidra_home/Ghidra" -name '*.jar' | tr '\n' ':')
classpath=${classpath%:}

outdir=$(mktemp -d)
trap 'rm -rf "$outdir"' EXIT

# -proc:none because the scripts use no annotation processors and Ghidra's jars
# carry some that would otherwise run.
"$javac" -nowarn -proc:none -cp "$classpath" -d "$outdir" "${scripts[@]}"

echo "ok: ${scripts[*]} compile against $ghidra_home"
