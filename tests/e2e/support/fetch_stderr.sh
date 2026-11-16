#!/bin/sh
# §FS-fetch.2.2: stdout stays snapshot data; stderr explains only a failed fetch.
# Check the selected config root and the sole, local argument before exercising the defect.
if [ "$#" -ne 1 ] || [ ! -f scripts/fetch ] || [ "$1" != "${1#*/}" ]; then
    echo 'fixture setup error: expected one local ID at the member root' >&2
    exit 93
fi
printf '# %s: A ticket\n\nFetched by the fixture.\n' "$1"
case "$1" in
    GR-2) echo 'fetch-ticket: successful progress stays quiet' >&2; exit 0 ;;
    GR-1) echo 'fetch-ticket: gdev-cli is not on PATH' >&2; exit 1 ;;
    GR-3) exit 1 ;;
    GR-4) printf 'fetch-ticket: credential expired\nrenew token: bad byte \377\n' >&2; exit 1 ;;
    GR-5) echo 'fetch-ticket: interrupted while contacting provider' >&2; kill -TERM "$$" ;;
    *) echo "fetch-ticket: $1: Issue Does Not Exist" >&2; exit 1 ;;
esac
