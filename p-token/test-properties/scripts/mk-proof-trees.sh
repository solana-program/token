#!/bin/bash
#
# Renders all proof trees given as arguments.
#
######################################################################

set -eu

SCRIPT_DIR=$(dirname $0)

PROOFS=$@

if [ -z "$PROOFS" ]; then
    echo "Missing argument (need at least one proof tree file)."
    exit 1
fi

# TODO check that all files actually exist

for proof in $PROOFS; do
    name="$(basename $proof)"
    printf '# Proof %-60s\n\n```\n' "${name%-full.txt}"
    python3 ${SCRIPT_DIR}/visualize_proof_tree.py "$proof"
    printf '\n```\n\n'
done
