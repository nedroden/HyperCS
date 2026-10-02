#!/usr/bin/env bash
# Format the API (rustfmt) and the web app (Prettier).
# Usage: ./format.sh [--check] [api|web]
#   --check  only verify formatting, change nothing (non-zero exit if a file needs formatting)
set -euo pipefail
cd "$(dirname "$0")"

check=false
target="all"
for arg in "$@"; do
    case "$arg" in
        --check) check=true ;;
        all | api | web) target="$arg" ;;
        *) echo "Usage: $0 [--check] [api|web]" >&2; exit 2 ;;
    esac
done

if [[ "$target" == "all" || "$target" == "api" ]]; then
    echo "==> api: rustfmt"
    if $check; then (cd api && cargo fmt --check); else (cd api && cargo fmt); fi
fi

if [[ "$target" == "all" || "$target" == "web" ]]; then
    echo "==> web: prettier"
    if $check; then (cd web && npm run --silent format:check); else (cd web && npm run --silent format >/dev/null); fi
fi

echo "Format OK"
