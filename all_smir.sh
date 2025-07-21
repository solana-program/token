#!/usr/bin/env bash
set -euxo pipefail

if [[ $# -lt 1 ]]; then
    printf "Usage: %s <output-filename-prefix>\n" "$0"
    exit 1;
fi

cd p-token
cargo clean
RUSTC=~/.stable-mir-json/release.sh cargo build
jq . ../target/debug/deps/pinocchio_token_program.smir.json > ../smir-out/${1}.smir.json
cargo clean
STABLE_MIR_OPTS=--dot RUSTC=~/.stable-mir-json/release.sh cargo build
mv ../target/debug/deps/pinocchio_token_program.smir.dot ../smir-out/${1}.smir.dot