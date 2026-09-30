#! /bin/bash
#
# Can this Ghidra decompile?
#
# Ghidra needs native binaries to decompile, and an official release does not
# carry them for every platform it supports. It ships Windows x86-64, Windows
# ARM64 and Linux x86-64; GettingStarted.md puts macOS, on either architecture,
# in the list "supported with user-built native binaries". So a Linux runner
# has had a complete Ghidra all along and a macOS one has never had one.
#
# Without the decompiler, `DecompInterface.decompileFunction` fails for every
# function, HighPCodeLifter.java skips each one, and the lift writes a
# parseable file with an empty functions array -- while analyzeHeadless exits 0
# and reports success (#54). That is what failed the macOS job on the same
# bytes the ubuntu one passed on (#126), and `lift` cannot yet tell anyone
# (#135).
#
# Which platforms get a prebuilt binary is the distribution's decision and can
# change, so this asks whether the binary is there rather than which runner it
# is on. It builds only when one is missing, which keeps a Linux runner
# exercising the same prebuilt binary its users download rather than one CI
# compiled for itself. The assertion at the end runs either way: a release that
# quietly stopped shipping Linux natives would fail here instead of lifting
# nothing.
#
# Building needs nothing installed beyond a JDK: the distribution supplies its
# own Gradle wrapper, and the runner images carry the C/C++ toolchains.
#
# Usage: ensure_ghidra_decompiler.sh [ghidra-install-dir]
#        defaults to $GHIDRA_INSTALL_DIR, then $GHIDRA_HOME.

set -euo pipefail

home="${1:-${GHIDRA_INSTALL_DIR:-${GHIDRA_HOME:-}}}"
if [ -z "$home" ]; then
  echo "No Ghidra directory given, and neither GHIDRA_INSTALL_DIR nor GHIDRA_HOME is set." >&2
  exit 2
fi
if [ ! -d "$home" ]; then
  echo "Not a directory: $home" >&2
  exit 2
fi

cd "$home"

# Ghidra prefers build/os/<platform>/ over the prebuilt os/<platform>/, so look
# in both rather than assuming which won. `decompile*`, because on Windows it
# is decompile.exe. `|| true`, because under `set -e` a failing find would exit
# before any of the reporting below.
natives() {
  find Ghidra/Features/Decompiler -type f -name 'decompile*' \
    -path '*os/*' -print 2>/dev/null | sort || true
}

if [ -z "$(natives)" ]; then
  echo "No decompiler binary in this distribution; building it."
  ( cd support/gradle && ./gradlew buildNatives )
else
  echo "The distribution already carries one; nothing to build."
fi

found=$(natives)
if [ -z "$found" ]; then
  echo "Ghidra has no decompiler binary, and building one did not produce it." >&2
  echo "Every lift from this installation would report success and find no functions." >&2
  find Ghidra/Features/Decompiler -type d -name os \
    -exec sh -c 'echo "  $1:"; ls -R "$1"' _ {} \; >&2 || true
  exit 1
fi

echo "Decompiler native binaries:"
echo "$found" | sed 's/^/  /'
