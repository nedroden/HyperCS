#!/usr/bin/env bash
# Run the linters for the API (clippy) and the web app (oxlint + steiger).
# Usage: ./lint.sh [api|web]   (default: both)
set -euo pipefail
cd "$(dirname "$0")"

target="${1:-all}"
case "$target" in
    all | api | web) ;;
    *) echo "Usage: $0 [api|web]" >&2; exit 2 ;;
esac

if [[ "$target" == "all" || "$target" == "api" ]]; then
    echo "==> api: cargo clippy"
    (cd api && cargo clippy --all-targets -- -D warnings)
fi

if [[ "$target" == "all" || "$target" == "web" ]]; then
    echo "==> web: oxlint + steiger"
    (cd web && npm run --silent lint)
fi

echo "Lint OK"
