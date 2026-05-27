#!/bin/bash
set -e

source /opt/gow/bash-lib/utils.sh

gow_log "Starting Wolf UI"
gow_log "Display backend: DISPLAY=${DISPLAY:-} GDK_BACKEND=${GDK_BACKEND} WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-} XDG_RUNTIME_DIR=${XDG_RUNTIME_DIR:-}"

source /opt/gow/launch-comp.sh

cd /opt/wolf-ui
launcher ./wolf-ui
