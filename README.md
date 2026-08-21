# Wolf UI Next

A lightweight, modern launcher for [Wolf](https://github.com/games-on-whales/wolf), shown when a Moonlight streaming session starts.

Wolf UI Next is a drop-in replacement for the [original Wolf UI](https://github.com/games-on-whales/wolf-ui), rebuilt with Rust and Dioxus.

![Wolf UI Next screenshot placeholder](https://placehold.co/1200x675/111827/e5e7eb?text=Wolf+UI+Next+Screenshot)

## Features

- Lower resource usage
- Modern, responsive UI for TVs, handhelds, and desktops
- Gamepad, keyboard, mouse, and touch navigation
- Drop-in replacement for the original Wolf UI container

## Configuration

In your Wolf `config.toml`, replace the existing Wolf UI app image with `ghcr.io/baz00k/wolf-ui-next:edge`. A minimal app configuration looks like this:

```toml
[[profiles.apps]]
title = "Wolf UI"
start_virtual_compositor = true

    [profiles.apps.runner]
    type = "docker"
    name = "Wolf-UI"
    image = "ghcr.io/baz00k/wolf-ui-next:edge"
    devices = []
    ports = []
    mounts = [
        "/var/run/wolf/wolf.sock:/var/run/wolf/wolf.sock"
    ]
    env = [
        "GOW_REQUIRED_DEVICES=/dev/input/event* /dev/dri/* /dev/nvidia*",
        "WOLF_SOCKET_PATH=/var/run/wolf/wolf.sock",
        "WOLF_UI_AUTOUPDATE=false"
    ]
```

Keep any additional devices, mounts, or Docker options from your current Wolf UI configuration. Restart Wolf after editing the file, then launch **Wolf UI** from Moonlight.

See the [Wolf configuration guide](https://games-on-whales.github.io/wolf/stable/user/configuration.html) for the complete app configuration format.

## Environment Variables

| Variable                   | Default                      | Purpose                                                                         |
| -------------------------- | ---------------------------- | ------------------------------------------------------------------------------- |
| `WOLF_SOCKET_PATH`         | `/var/run/wolf/wolf.sock`    | Path to Wolf's Unix socket.                                                     |
| `WOLF_API_BASE_URL`        | unset                        | Use Wolf's HTTP API instead of the Unix socket, for example `http://wolf:8080`. |
| `WOLF_UI_SETTINGS_ENABLED` | `false`                      | Show the settings area. Accepts `1`, `true`, `yes`, or `on`.                    |
| `WOLF_UI_AUTOUPDATE`       | `false`                      | Allow automatic image updates at startup. Accepts `1`, `true`, `yes`, or `on`.  |
| `WOLF_SESSION_ID`          | provided by Wolf             | Identifies the current session and enables session controls.                    |
| `WOLF_VIDEO_BUFFER_CAPS`   | provided by Wolf             | Carries the current stream's video buffer capabilities when creating a lobby.   |
| `RUST_LOG`                 | `wolf_ui=info,wolf_api=info` | Controls application log filtering.                                             |

## Development

Linux development needs [rustup](https://rustup.rs/), Docker, [mise](https://mise.jdx.dev/), and these Debian/Ubuntu packages:

```bash
sudo apt-get install build-essential ca-certificates curl jq libayatana-appindicator3-dev libgtk-3-dev libudev-dev librsvg2-dev libssl-dev libwebkit2gtk-4.1-dev libxdo-dev pkg-config
mise install
mise run dev
mise run check
```

`mise run dev` starts Wolf and runs the UI through Dioxus. See [development.md](docs/development.md) for external Wolf and production-container testing.

## Related Projects

- [Wolf](https://github.com/games-on-whales/wolf)
- [Original Wolf UI](https://github.com/games-on-whales/wolf-ui)
