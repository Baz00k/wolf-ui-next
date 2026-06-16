# syntax=docker/dockerfile:1.7

ARG BASE_APP_IMAGE=ghcr.io/games-on-whales/base-app:edge
ARG RUST_VERSION=1.96.0
ARG DIOXUS_CLI_VERSION=0.7.9

FROM rust:${RUST_VERSION}-trixie AS chef

ARG DIOXUS_CLI_VERSION

ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:/.cargo/bin:${PATH}

RUN <<_INSTALL_BUILD_DEPS
set -e
apt-get update -y
apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    libayatana-appindicator3-dev \
    libgtk-3-dev \
    libudev-dev \
    librsvg2-dev \
    libssl-dev \
    libwebkit2gtk-4.1-dev \
    libxdo-dev \
    pkg-config
rm -rf /var/lib/apt/lists/*
_INSTALL_BUILD_DEPS

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    <<_INSTALL_DX
set -e
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
cargo binstall dioxus-cli --version "${DIOXUS_CLI_VERSION}" --root /.cargo -y --force
cargo binstall cargo-chef --root /.cargo -y --force
_INSTALL_DX

WORKDIR /app

FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook \
        --profile desktop-release \
        --no-default-features \
        --features desktop \
        --package wolf-ui \
        --recipe-path recipe.json

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    <<_BUILD_APP
set -e
dx build --desktop --release --package wolf-ui --locked
mkdir -p /app/dist
cp -a target/dx/wolf-ui/release/linux/app /app/dist/app
test -x /app/dist/app/wolf-ui
_BUILD_APP

FROM ${BASE_APP_IMAGE} AS runtime

ENV DEBIAN_FRONTEND=noninteractive

RUN <<_INSTALL_RUNTIME_DEPS
set -e
apt-get update -y
apt-get install -y --no-install-recommends \
    ca-certificates \
    libayatana-appindicator3-1 \
    libgtk-3-0t64 \
    libudev1 \
    librsvg2-2 \
    libwebkit2gtk-4.1-0 \
    libxdo3
rm -rf /var/lib/apt/lists/*
_INSTALL_RUNTIME_DEPS

ENV PUID=0 \
    PGID=0 \
    UNAME=root \
    GDK_BACKEND=wayland

COPY --from=builder /app/dist/app /opt/wolf-ui
RUN ln -s /opt/wolf-ui/wolf-ui /usr/local/bin/wolf-ui
COPY --chmod=777 container-overlay/ /
RUN <<_WRAP_WEBKIT_HELPERS
set -e
for helper in /usr/lib/*/webkit2gtk-4.1/WebKitWebProcess \
              /usr/lib/*/webkit2gtk-4.1/WebKitNetworkProcess \
              /usr/lib/*/webkit2gtk-4.1/WebKitGPUProcess; do
    if [ -x "$helper" ] && [ ! -e "$helper.real" ]; then
        mv "$helper" "$helper.real"
        cp /usr/local/bin/webkit-helper-wrapper.sh "$helper"
        chmod 755 "$helper"
    fi
done
_WRAP_WEBKIT_HELPERS

ARG IMAGE_SOURCE
ARG IMAGE_REVISION
ARG IMAGE_VERSION
LABEL org.opencontainers.image.source=$IMAGE_SOURCE \
      org.opencontainers.image.revision=$IMAGE_REVISION \
      org.opencontainers.image.version=$IMAGE_VERSION
