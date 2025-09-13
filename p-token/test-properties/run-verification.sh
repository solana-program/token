#!/bin/bash
#
# Run formal verification with runtime-verification feature enabled
# This script sets up and runs the verification process for p-token
#
# Usage:
#   ./run-verification.sh [test-name]  # Run specific test
#   ./run-verification.sh -a           # Run all tests
#
######################################################################

set -euo pipefail

# Change to script directory
SCRIPT_DIR="$(dirname "$0")"
cd "$SCRIPT_DIR"

# First, ensure we build with runtime-verification feature
echo "Building p-token with runtime-verification feature..."
cd ..
cargo clean
export RUSTC=$PWD/test-properties/mir-semantics/deps/.stable-mir-json/release.sh

# Build with runtime-verification feature to get test functions
cargo build --features runtime-verification

cd test-properties

# Link SMIR files
echo "Linking SMIR files..."
SMIRS=$(ls ../../target/debug/deps/*smir.json | sort)
mkdir -p artefacts/
uv --project mir-semantics/kmir run -- kmir link ${SMIRS} -o artefacts/p-token.smir.json

# Now run the proofs
echo "Running verification proofs..."
./run-proofs.sh "$@"