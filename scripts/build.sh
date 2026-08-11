#!/usr/bin/env bash
set -euo pipefail

export ARTIFACT_NAME="che-$1"
export CHE_GEN_COMPLETIONS=1
export YAZI_GEN_COMPLETIONS=1

# Build the target
git config --global --add safe.directory "*"
cargo build --release --locked --target "$1"

# Copy the binaries to a known location
mkdir -p "target/release"
if [[ "$1" == *windows* ]]; then
	cp "target/$1/release/ch.exe" "target/release/ch.exe"
	cp "target/$1/release/che.exe" "target/release/che.exe"
else
	cp "target/$1/release/ch" "target/release/ch"
	cp "target/$1/release/che" "target/release/che"
fi

# Create the artifact
mkdir -p "$ARTIFACT_NAME/completions"
if [[ "$1" == *windows* ]]; then
	cp "target/release/ch.exe" "$ARTIFACT_NAME"
	cp "target/release/che.exe" "$ARTIFACT_NAME"
else
	cp "target/release/ch" "$ARTIFACT_NAME"
	cp "target/release/che" "$ARTIFACT_NAME"
fi

if [ -d "yazi-cli/completions" ]; then
	cp yazi-cli/completions/* "$ARTIFACT_NAME/completions" 2>/dev/null || true
fi
if [ -d "yazi-boot/completions" ]; then
	cp yazi-boot/completions/* "$ARTIFACT_NAME/completions" 2>/dev/null || true
fi
cp README.md LICENSE "$ARTIFACT_NAME"

# Zip the artifact
if ! command -v zip &> /dev/null; then
	apt-get update && apt-get install -yq zip 2>/dev/null || true
fi
zip -r "$ARTIFACT_NAME.zip" "$ARTIFACT_NAME"
