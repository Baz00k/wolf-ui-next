#!/bin/bash
set -e

source /opt/gow/bash-lib/utils.sh

gow_log "Starting Wolf UI"

gow_log "[DEBUG-display] uid=$(id -u) gid=$(id -g) user=$(id -un 2>/dev/null || true) groups=$(id -Gn 2>/dev/null || true)"
if [ "${DISPLAY+x}" = "x" ]; then
    gow_log "[DEBUG-display] DISPLAY is set to '${DISPLAY}'"
else
    gow_log "[DEBUG-display] DISPLAY is unset"
fi
if [ "${GDK_BACKEND+x}" = "x" ]; then
    gow_log "[DEBUG-display] GDK_BACKEND is set to '${GDK_BACKEND}'"
else
    gow_log "[DEBUG-display] GDK_BACKEND is unset"
fi
if [ "${WAYLAND_DISPLAY+x}" = "x" ]; then
    gow_log "[DEBUG-display] WAYLAND_DISPLAY is set to '${WAYLAND_DISPLAY}'"
else
    gow_log "[DEBUG-display] WAYLAND_DISPLAY is unset"
fi
if [ "${XDG_RUNTIME_DIR+x}" = "x" ]; then
    gow_log "[DEBUG-display] XDG_RUNTIME_DIR is set to '${XDG_RUNTIME_DIR}'"
else
    gow_log "[DEBUG-display] XDG_RUNTIME_DIR is unset"
fi

if [ -n "${XDG_RUNTIME_DIR:-}" ]; then
    gow_log "[DEBUG-display] runtime dir: $(ls -ld "${XDG_RUNTIME_DIR}" 2>&1 || true)"
    gow_log "[DEBUG-display] runtime dir entries: $(ls -la "${XDG_RUNTIME_DIR}" 2>&1 | tr '\n' ';' || true)"
fi
if [ -n "${XDG_RUNTIME_DIR:-}" ] && [ -n "${WAYLAND_DISPLAY:-}" ]; then
    wayland_socket="${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}"
    gow_log "[DEBUG-display] wayland socket: $(ls -l "${wayland_socket}" 2>&1 || true)"
    if [ -S "${wayland_socket}" ]; then
        gow_log "[DEBUG-display] wayland socket test: socket exists"
    else
        gow_log "[DEBUG-display] wayland socket test: socket missing or not a socket"
    fi
fi

for helper in /usr/lib/*/webkit2gtk-4.1/WebKitWebProcess \
              /usr/lib/*/webkit2gtk-4.1/WebKitNetworkProcess \
              /usr/lib/*/webkit2gtk-4.1/WebKitGPUProcess; do
    if [ -e "${helper}" ]; then
        gow_log "[DEBUG-display] webkit helper ${helper}: $(ls -l "${helper}" 2>&1 || true)"
    fi
done

gow_log "[DEBUG-display] wolf-ui binary: $(ls -l /opt/wolf-ui/wolf-ui 2>&1 || true)"
gow_log "[DEBUG-display] app dir entries: $(ls -la /opt/wolf-ui 2>&1 | tr '\n' ';' || true)"
gow_log "[DEBUG-display] selected env: $(env | sort | grep -E '^(DISPLAY|GDK_BACKEND|WAYLAND_DISPLAY|XDG_RUNTIME_DIR|WEBKIT|GTK|GIO|NO_AT_BRIDGE|LD_LIBRARY_PATH|XDG_SESSION_TYPE|XDG_CURRENT_DESKTOP)=' | tr '\n' ';' || true)"

source /opt/gow/launch-comp.sh

cd /opt/wolf-ui
launcher ./wolf-ui
