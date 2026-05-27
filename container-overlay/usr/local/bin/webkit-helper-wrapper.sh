#!/bin/bash

log() {
    printf '[DEBUG-webkit-helper] %s\n' "$*" >&2
}

log "helper=$0 args=$*"
log "uid=$(id -u) gid=$(id -g) user=$(id -un 2>/dev/null || true) groups=$(id -Gn 2>/dev/null || true)"

if [ "${DISPLAY+x}" = "x" ]; then
    log "DISPLAY is set to '${DISPLAY}'"
else
    log "DISPLAY is unset"
fi
if [ "${GDK_BACKEND+x}" = "x" ]; then
    log "GDK_BACKEND is set to '${GDK_BACKEND}'"
else
    log "GDK_BACKEND is unset"
fi
if [ "${WAYLAND_DISPLAY+x}" = "x" ]; then
    log "WAYLAND_DISPLAY is set to '${WAYLAND_DISPLAY}'"
else
    log "WAYLAND_DISPLAY is unset"
fi
if [ "${XDG_RUNTIME_DIR+x}" = "x" ]; then
    log "XDG_RUNTIME_DIR is set to '${XDG_RUNTIME_DIR}'"
else
    log "XDG_RUNTIME_DIR is unset"
fi

if [ -n "${XDG_RUNTIME_DIR:-}" ]; then
    log "runtime dir: $(ls -ld "${XDG_RUNTIME_DIR}" 2>&1 || true)"
fi
if [ -n "${XDG_RUNTIME_DIR:-}" ] && [ -n "${WAYLAND_DISPLAY:-}" ]; then
    wayland_socket="${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}"
    log "wayland socket: $(ls -l "${wayland_socket}" 2>&1 || true)"
    if [ -S "${wayland_socket}" ]; then
        log "wayland socket test: socket exists"
    else
        log "wayland socket test: socket missing or not a socket"
    fi
fi

log "selected env: $(env | sort | grep -E '^(DISPLAY|GDK_BACKEND|WAYLAND_DISPLAY|XDG_RUNTIME_DIR|WEBKIT|GTK|GIO|NO_AT_BRIDGE|LD_LIBRARY_PATH|XDG_SESSION_TYPE|XDG_CURRENT_DESKTOP)=' | tr '\n' ';' || true)"

exec "$0.real" "$@"
