#!/usr/bin/env bash
set -euo pipefail

if ! pkg-config --exists webkit2gtk-4.1; then
  echo "Missing webkit2gtk-4.1 development files. Install the packages listed in README.md." >&2
  exit 1
fi

exec dx serve
