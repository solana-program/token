#!/bin/bash
#
# Setup for running p-token property tests with kmir
# * checks out submodule mir-semantics (recursively also
#   including stable-mir-json)
# * builds stable-mir-json and mir-semantics
# * builds p-token with stable-mir-json and links SMIR JSON
#
# This should be run just once when making changes to mir-semantics.
#
# Usage:
#   ./setup.sh                  # Normal execution, including refreshing submodules
#   ./setup.sh --skip-submodules  # Skip refreshing submodules
#
# After running it, one can use kmir with
#   `uv --project mir-semantics/kmir run -- kmir ...`
######################################################################

set -xeuo pipefail

# 解析命令行参数
SKIP_SUBMODULES=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-submodules)
            SKIP_SUBMODULES=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --skip-submodules    Skip refreshing git submodules"
            echo "  -h, --help          Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# run from the script's directory
SCRIPT_DIR="$(realpath $(dirname "$0"))"
cd "${SCRIPT_DIR}"

# refresh/check out submodules (unless skipped)
if [ "$SKIP_SUBMODULES" = false ]; then
    echo "Refreshing git submodules..."
    git submodule update --init --recursive
    git submodule status --recursive
else
    echo "Skipping git submodule refresh..."
fi

# if any changes were already made, keep them. Otherwise, apply a
# workaround to avoid confusion with the token workspace.
if [ -z "$(cd mir-semantics && git status --porcelain)" ]; then
    printf "\n\n# avoid workspace confusion in token repo\n[workspace]\n" \
           >> mir-semantics/deps/stable-mir-json/Cargo.toml
fi
# build mir-semantics and stable-mir-json
make -C mir-semantics stable-mir-json build CARGO_BUILD_OPTS=--release

export RUSTC=$PWD/mir-semantics/deps/.stable-mir-json/release.sh
${RUSTC} --version

# export STABLE_MIR_OPTS="-Zno-codegen"

# build p-token with stable-mir-json
# NB deletes all prior token build artefacts
cd .. # p-token
cargo clean && cargo build --features runtime-verification
cd test-properties
SMIRS=$(ls ../../target/debug/deps/*smir.json | sort)

ls  $SMIRS

# link all SMIR JSON and store in artefacts directory
mkdir -p artefacts/
uv --project mir-semantics/kmir run -- kmir link ${SMIRS} -o artefacts/p-token.smir.json
