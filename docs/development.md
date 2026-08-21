# Development

```bash
mise install
mise run dev
mise run check
```

`mise run dev` starts Wolf in Docker and Wolf UI through Dioxus. The development Wolf has access to the host Docker daemon.

Edit `dev/wolf/config.toml`, then run `mise run wolf-reset` to recreate Wolf state. Use `mise run wolf-logs` for startup errors.

## Other Wolf instances

```bash
WOLF_SOCKET_PATH=/path/to/wolf.sock mise run serve
WOLF_API_BASE_URL=http://host:port mise run serve
```

## Production container

Production testing requires a GPU, `/dev/uinput`, `/dev/uhid`, and Moonlight:

```bash
mise run prod
```

Add the development host in Moonlight, pair it, and launch **Wolf UI**. Override the GPU with `WOLF_RENDER_NODE=/dev/dri/renderD129 mise run prod`.

Stop Wolf with `mise run wolf-down`. Run `mise run update-schema` when updating the pinned Wolf image.
