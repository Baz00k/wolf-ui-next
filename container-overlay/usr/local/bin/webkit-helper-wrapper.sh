#!/bin/bash
set -e

export GDK_BACKEND=wayland

exec "$0.real" "$@"
