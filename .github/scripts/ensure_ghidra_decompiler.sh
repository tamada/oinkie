#! /bin/bash
#
# Can this Ghidra decompile?
#
# Ghidra needs native binaries to decompile, and an official release does not
# carry them for every platform it supports. It ships Windows x86-64, Windows
# ARM64 and Linux x86-64; GettingStarted.md puts macOS, on either architecture,
# in the list "supported with user-built native binaries".
#
# Without one for the platform it is running on, `DecompInterface` fails for
# every function, HighPCodeLifter.java skips each one, and the lift writes a
# parseable file with an empty functions array -- while analyzeHeadless exits 0
# and reports success (#54). That is what failed the macOS job on the same
# bytes the ubuntu one passed on (#126), and `lift` cannot yet tell anyone
# (#135).
#
# This builds unconditionally, and that is the point rather than a shortcut.
# The version before it looked for an existing binary and skipped the build
# when it found one -- and on macOS it found `os/linux_x86_64/decompile` and
# `os/win_x86_64/decompile.exe`, declared the distribution complete, and left
# the lift as empty as before. Asking "is there a binary" is not the question;
# the question is "is there one for this machine", and answering that needs a
# map from uname to Ghidra's platform names, which is one more thing to get
# wrong. Building needs no such map: gradle knows what host it is on.
#
# It costs 88 seconds, measured on the macOS runner (1m26s of gradle, plus the
# wrapper download), in a workflow that takes about six minutes. Which
# platforms get a prebuilt binary is the distribution's decision and can
# change; this does not depend on that decision being what it is today.
#
# Nothing has to be installed for it: the distribution supplies its own Gradle
# wrapper, the JDK is a Ghidra requirement anyway, and the runner images carry
# the C/C++ toolchains.
#
# Usage: ensure_ghidra_decompiler.sh [ghidra-install-dir]
#        defaults to $GHIDRA_INSTALL_DIR, then $GHIDRA_HOME.

set -euo pipefail

home="${1:-${GHIDRA_INSTALL_DIR:-${GHIDRA_HOME:-}}}"
if [[ -z "$home" ]]; then
  echo "No Ghidra directory given, and neither GHIDRA_INSTALL_DIR nor GHIDRA_HOME is set." >&2
  exit 2
fi
if [[ ! -d "$home" ]]; then
  echo "Not a directory: $home" >&2
  exit 2
fi

cd "$home"
( cd support/gradle && ./gradlew buildNatives )

# Only under build/os/, never the prebuilt os/. GettingStarted.md: a build
# leaves its binaries "in the relevant modules' `build/os/<platform>/`
# subdirectories, which Ghidra will prefer to any existing pre-built native
# binaries in the `os/<platform>/` subdirectories". So this directory holds
# what was built for *this* machine, which is the thing being asserted --
# whereas os/ holds whatever the distribution shipped for other people's.
#
# `decompile*`, because on Windows it is decompile.exe. `|| true`, because
# under `set -e` a failing find would exit before the reporting below.
built=$(find Ghidra/Features/Decompiler/build/os -type f -name 'decompile*' \
          -print 2>/dev/null | sort || true)

if [[ -z "$built" ]]; then
  echo "buildNatives finished without producing a decompiler for this platform." >&2
  echo "Every lift from this installation would report success and find no functions." >&2
  find Ghidra/Features/Decompiler -type d -name os \
    -exec sh -c 'echo "  $1:"; ls -R "$1"' _ {} \; >&2 || true
  exit 1
fi

echo "Decompiler built for this platform:"
echo "$built" | sed 's/^/  /'
