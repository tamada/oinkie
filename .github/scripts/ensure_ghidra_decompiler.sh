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
# and reports success.
#
# This builds unconditionally, and that is the point rather than a shortcut.
# Looking for an existing binary is the wrong question: a distribution carries
# other platforms' binaries (`os/linux_x86_64/decompile`,
# `os/win_x86_64/decompile.exe`), and finding those says nothing about this
# machine. "Is there one for this machine" needs a map from uname to Ghidra's
# platform names, which is one more thing to get wrong; building needs no such
# map, since gradle knows what host it is on.
#
# It costs about 90 seconds on the macOS runner, in a workflow that takes
# about six minutes. Which platforms get a prebuilt binary is the
# distribution's decision and can change; this does not depend on it.
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
