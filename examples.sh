#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<EOF
Usage: $(basename "$0") [--debug] <example>

Options:
  --debug    Enable debug features

Examples:
  walking_squad

EOF
  exit 0
}

# ── Parse flags ───────────────────────────────────────────────
DEBUG=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --debug)
      DEBUG=1
      shift
      ;;
    help|--help|-h)
      usage
      ;;
    -*)
      echo "Error: unknown option '$1'" >&2
      usage
      ;;
    *)
      break
      ;;
  esac
done

# ── Require exactly one argument ──────────────────────────────
if [[ $# -ne 1 ]]; then
  echo "Error: expected 1 argument, got $#" >&2
  usage
fi

# ── Build cargo extra args ────────────────────────────────────
CARGO_EXTRA_ARGS=""
if [[ $DEBUG -eq 1 ]]; then
  CARGO_EXTRA_ARGS="--features debug"
fi

case "$1" in
  walking_squad)
      export WORLD_WIDTH=1000
      export WORLD_HEIGHT=1000
      export REGION_WIDTH=100
      export REGION_HEIGHT=100
      cargo run --bin example_walking_squad $CARGO_EXTRA_ARGS
    ;;
  *)
    echo "Error: unknown example '$1'" >&2
    usage
    ;;
esac
