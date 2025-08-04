#!/bin/bash
#
# Run all start symbols given as arguments (or read them from tests.md
# table if -a given) with given run options (-o) and timeout (-t).
# Options and defaults:
#   -t NUM   : timeout in seconds (default 300)
#   -o STRING: prove-rs options. Default "--max-iterations 20 --max-depth 100 "
#   -a       : run all start symbols from table in `tests.md` (1st column)
#
# Always runs verbosely, always reloads, always uses artefacts/proof
# as proof directory
#
#######################################################################

ALL_NAMES=$(sed -n -e 's/^| \([a-zA-Z0-9:_]*\) *|.*/\1/p' tests.md)

TIMEOUT=300
PROVE_OPTS="--max-iterations 20 --max-depth 100"

while getopts ":t:o:a" opt; do
    case $opt in
        t)
            TIMEOUT=$OPTARG
            ;;
        o)
            PROVE_OPTS=$OPTARG
            ;;
        a)
            TESTS=${ALL_NAMES}
            ;;
        \?)
            echo "[ERROR] Invalid option -$OPTARG." 1>&2
            echo 1>&2
            head -14 $0 1>&2
            exit 1
            ;;
    esac
done
shift $((OPTIND-1))
if [ -z "$TESTS" ]; then
    if [ -z "$@" ]; then
        echo "[ERROR] No test function names given. Use -a or provide at least one name." 1>&2
        exit 2
    fi
    TESTS=$@
fi

set -u

echo "Running tests ${TESTS} with options '$PROVE_OPTS' and timeout $TIMEOUT"

for name in $TESTS; do
    echo "============================== $name ============================"
    timeout --preserve-status -v ${TIMEOUT} \
            uv --project mir-semantics/kmir run -- \
            kmir prove-rs --smir artefacts/p-token.smir.json \
            --proof-dir artefacts/proof --reload --verbose --start-symbol $name ${PROVE_OPTS}
    echo "==========================================================================="
done
