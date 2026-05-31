#!/usr/bin/env bash
# Reads NDJSON requests on stdin, echoes a fixed `{"kind":"ok"}` for each.
set -eu
while IFS= read -r line; do
    if [[ -z "$line" ]]; then
        continue
    fi
    echo '{"kind":"ok"}'
done
