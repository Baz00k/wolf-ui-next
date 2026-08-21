#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
docker compose -f "${repo_root}/dev/compose.yml" down
rm -rf "${repo_root}/dev/.state"
echo "Wolf development state removed; the next start will reseed it."
