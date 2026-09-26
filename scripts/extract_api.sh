#!/bin/sh
# Regenerate content/api.json from the apsis-rs sources.
#
# The catalog is what the service shows as the interface documentation of
# `apsis-rs`. It is generated once and committed, so that building the Rust
# binary does not depend on the apsis-rs checkout or on Python.
#
# Usage, from the apsis-book directory:
#
#     scripts/extract_api.sh
#
# Set APSIS_RS_WORKSPACE to point at a checkout outside the default
# location, which is ../apsis-rs relative to this project.

set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec python3 "$here/extract_api.py"
