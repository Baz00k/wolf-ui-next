#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
socket_path="/tmp/sockets/wolf.sock"
render_node="${WOLF_RENDER_NODE:-/dev/dri/renderD128}"
compose=(
  docker compose
  -f "${repo_root}/dev/compose.yml"
  -f "${repo_root}/dev/compose.prod.yml"
)

for path in /dev/dri /dev/uinput /dev/uhid /run/udev; do
  if [[ ! -e "${path}" ]]; then
    echo "Production testing requires ${path}." >&2
    exit 1
  fi
done

if [[ ! -e "${render_node}" ]]; then
  echo "Production testing requires render node ${render_node}." >&2
  exit 1
fi

mise run image
bash "${repo_root}/dev/scripts/wolf-up.sh"
"${compose[@]}" up -d --wait --wait-timeout 60

for _ in {1..30}; do
  if [[ -S "${socket_path}" ]]; then
    "${compose[@]}" exec -T wolf chmod 666 /tmp/sockets/wolf.sock >/dev/null 2>&1 || true
    if curl --fail --silent --max-time 2 \
      --unix-socket "${socket_path}" \
      http://localhost/api/v1/openapi-schema >/dev/null; then
      echo "Wolf production test environment is ready."
      echo "Open Moonlight, add this host, pair it, and launch 'Wolf UI'."
      echo "Use 'mise run wolf-logs' for pairing details and 'mise run wolf-down' when finished."
      exit 0
    fi
  fi
  sleep 1
done

echo "Wolf API did not become ready. Inspect it with 'mise run wolf-logs'." >&2
exit 1
