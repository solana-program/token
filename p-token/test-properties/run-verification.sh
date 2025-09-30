#!/bin/bash
#
# Run formal verification with runtime-verification feature enabled
# This script sets up and runs the verification process for p-token
#
# Usage:
#   ./run-verification.sh [test-name]       # Run specific test
#   ./run-verification.sh -a                # Run all tests
#   ./run-verification.sh --multisig [test] # Run with multisig feature enabled
#   ./run-verification.sh --multisig -a     # Run with multisig feature enabled and run all tests
#
######################################################################

set -euo pipefail

# Parse command line arguments
FEATURES="runtime-verification"
while [[ $# -gt 0 ]]; do
    case $1 in
        --multisig)
            FEATURES="runtime-verification,multisig"
            shift
            ;;
        *)
            break
            ;;
    esac
done

# Change to script directory
SCRIPT_DIR="$(dirname "$0")"
cd "$SCRIPT_DIR"

# First, ensure we build with runtime-verification feature
echo "Building p-token with features: $FEATURES"
cd ..
cargo clean
export RUSTC=$PWD/test-properties/mir-semantics/deps/.stable-mir-json/release.sh

# Build with runtime-verification feature to get test functions
cargo build --features "$FEATURES"

cd test-properties

# Link SMIR files
echo "Linking SMIR files..."
SMIRS=$(ls ../../target/debug/deps/*smir.json | sort)
mkdir -p artefacts/
uv --project mir-semantics/kmir run -- kmir link ${SMIRS} -o artefacts/p-token.smir.json

# Now run the proofs
echo "Running verification proofs..."
./run-proofs.sh "$@"