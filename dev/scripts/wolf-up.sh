#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
state_dir="${repo_root}/dev/.state"
socket_path="/tmp/sockets/wolf.sock"
compose=(docker compose -f "${repo_root}/dev/compose.yml")

if ! docker info >/dev/null 2>&1; then
  echo "Docker is unavailable. Start Docker, then rerun 'mise run wolf-up'." >&2
  exit 1
fi

mkdir -p "${state_dir}/cfg" "${state_dir}/icons"
chmod 700 "${state_dir}" "${state_dir}/cfg" "${state_dir}/icons"

if [[ ! -e "${state_dir}/cfg/config.toml" ]]; then
  cp "${repo_root}/dev/wolf/config.toml" "${state_dir}/cfg/config.toml"
  for source in "${repo_root}"/dev/wolf/icons/*.png.b64; do
    destination="${state_dir}/icons/$(basename "${source%.b64}")"
    base64 --decode "${source}" > "${destination}"
  done
  echo "Seeded Wolf state in dev/.state."
fi

"${compose[@]}" up -d --wait --wait-timeout 60

for _ in {1..30}; do
  if [[ -S "${socket_path}" ]]; then
    # Wolf commonly starts as root; keep the development socket usable by the host.
    "${compose[@]}" exec -T wolf chmod 666 /tmp/sockets/wolf.sock >/dev/null 2>&1 || true
    if curl --fail --silent --max-time 2 \
      --unix-socket "${socket_path}" \
      http://localhost/api/v1/openapi-schema >/dev/null; then
      echo "Wolf API is ready at ${socket_path}."
      exit 0
    fi
  fi
  sleep 1
done

echo "Wolf API did not become ready. Inspect it with 'mise run wolf-logs'." >&2
exit 1
