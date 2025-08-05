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
# After running it, one can use kmir with
#   `uv --project mir-semantics/kmir run -- kmir ...`
######################################################################

set -eu

# run from the script's directory
SCRIPT_DIR="$(realpath $(dirname "$0"))"
cd "${SCRIPT_DIR}"

# refresh/check out submodules
git submodule update --init --recursive
git submodule status --recursive

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

# build p-token with stable-mir-json
# NB deletes all prior token build artefacts
cd .. # p-token
cargo clean && cargo build
cd test-properties
SMIRS=$(ls ../../target/debug/deps/*smir.json | sort)

ls  $SMIRS

# link all SMIR JSON and store in artefacts directory
mkdir -p artefacts/
uv --project mir-semantics/kmir run -- kmir link ${SMIRS} -o artefacts/p-token.smir.json
