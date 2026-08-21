#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
schema="${repo_root}/crates/wolf-api/openapi/wolf.openapi.json"
temporary="$(mktemp)"
trap 'rm -f "${temporary}"' EXIT

curl --fail --silent --show-error \
  --unix-socket "${repo_root}/dev/.state/run/wolf.sock" \
  http://localhost/api/v1/openapi-schema |
  jq . > "${temporary}"
cat "${temporary}" > "${schema}"

cd "${repo_root}"
cargo run --locked -p wolf-api-gen
mise run check
echo "Wolf API schema and generated types updated; review the diff."
