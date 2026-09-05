#! /bin/bash

# Ghidra Settings
if [[ -n "$GHIDRA_HOME" ]]; then
    echo "Using Ghidra path from environment variable: $GHIDRA_HOME"
else
    GHIDRA_HOME="/opt/homebrew/opt/ghidra/libexec"
fi

GHIDRA_HEADLESS="${GHIDRA_HOME}/support/analyzeHeadless"
SCRIPT_DIR="$(pwd)/assets/lifters/ghidra/scripts"
TARGET_BIN=$(realpath "$1") # The target binary for binary lifting, passed as an argument to the script
PROJ_NAME="$(basename $TARGET_BIN)"
PROJ_DIR="$(echo "$TARGET_BIN" | sed 's|executables|ghidra|g')/"
DEST_DIR="$(echo "$TARGET_BIN" | sed 's|executables|pcodes|g')/"

if [ -z "$TARGET_BIN" ]; then
    echo "Usage: $0 <target_binary>"
    exit 1
fi

# creating the temporary project directory.
mkdir -p "$PROJ_DIR"

# Execute Ghidra headless analysis
#   -import: Import the binary
#   -postScript: Run Java script after analysis
#   -deleteProject: Delete the project after completion
time $GHIDRA_HEADLESS "$PROJ_DIR" "$PROJ_NAME" \
    -import "$TARGET_BIN" \
    -scriptPath "$SCRIPT_DIR" \
    -postScript "$SCRIPT_DIR/HighPCodeLifter.java"
    # -deleteProject \

echo "Generated PCode for ${PROJ_NAME}.json"
mkdir -p $(dirname $DEST_DIR)
mv ${PROJ_NAME}.json $(dirname $DEST_DIR)/${PROJ_NAME}.json

# deleting the temporary project directory
# rm -rf "$PROJ_DIR"
