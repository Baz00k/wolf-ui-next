# Missing UI audio investigation

Investigated **2026-10-01**, at checkout `1469e70ef4b9034c099f5579b749070f770fcaac`.
Initial investigation used disposable test containers only. After the user confirmed `image = "ghcr.io/baz00k/wolf-ui-next:edge-fedora"`, the Fedora runtime package list was patched as described below. Application code, sound gains, and user configuration were not changed.

## Summary

**The published Fedora runtime has a reproducible audio dependency bug.**
The Dockerfile at the investigated published revision installed `webkit2gtk4.1` with weak dependencies disabled but did not explicitly install `gstreamer1-plugins-good`.
Fedora only recommends that package. Its omission removes the WebAudio output/discovery and decoding elements `pulsesink`, `autoaudiosink`, and `deinterleave`. [F1][F1], [F2][F2], [F3][F3]

Installing **only `gstreamer1-plugins-good`** in the disposable Fedora container fixed all three missing elements, restored Ogg decoding, and produced nonzero audio on Wolf's PulseAudio monitor. The Ubuntu image already includes Good and produced audio without a pointer/keyboard gesture. The user subsequently supplied a Wolf app configuration selecting the affected `edge-fedora` tag. The exact digest deployed on their host was not supplied, so local reproduction does not replace validation of their live session.

## Applied fix

- `Dockerfile` now explicitly installs `gstreamer1-plugins-good` in the Fedora branch, without enabling weak dependencies or adding another audio server.
- The Fedora image build now fails if any of `autoaudiosink`, `deinterleave`, `pulsesink`, `oggdemux`, or `vorbisdec` is unavailable. These checks run at build time only; no runtime polling or startup overhead was added.
- The existing Docker-build workflow already builds the Fedora variant, so these factory assertions also run for that variant in CI. No workflow or Wolf configuration change is required.
- The published image tag is **not** changed by editing this checkout. The fix needs to be built/published and the user's app container recreated before that host receives it.
- **Post-patch validation:** extracted the exact updated `_INSTALL_RUNTIME_DEPS` Dockerfile block and built it on the pinned, unmodified published Fedora UI image as `wolf-ui-next:audio-fix-check`. This isolates the changed dependency stage without recompiling the unchanged Rust application. The build passed all five factory assertions. In the resulting image, all three bundled Ogg assets decoded and passed through `deinterleave` successfully using GStreamer. The earlier same-package WebAudio experiment additionally confirmed PulseAudio output.
- `mise run check` was attempted again after the patch but stopped at `cargo: not found`. A full source-to-production-image build and full Moonlight session remain unverified here; the corrected image has not been published.

## Follow-up: did Wolf move to per-container PulseAudio?

**Not per app container.** There was a real migration on **2026-06-07** (Wolf PR #422): the shared PulseAudio daemon moved from the standalone `WolfPulseAudio` sidecar into the **Wolf container**, supervised alongside Wolf. External-server support and the sidecar fallback remain. [H1][H1]

Separately, on **2026-02-07**, PR #347 added a router for applications that ignore `PULSE_SINK`. It uses `application.process.host` → Docker container's runner/session ID → `virtual_sink_<id>`. This is per-session/lobby **routing on the shared daemon**, not one daemon per application. Lobby runners use a lobby ID for this purpose. [H2][H2], [H3][H3], [H4][H4]

The locally pinned Wolf image identifies revision `7f416ec0f26bb4d95218af806cd1181fe5434598` (**2026-08-08**, PR #472). Its live processes confirmed one PulseAudio process alongside Wolf. Upstream `stable` resolved to `facb8e0ccd22fd6091cd904741e4dcf895e47b1b` (**2026-09-29**, PR #505). Comparing these revisions showed no changes to embedded PulseAudio startup/supervision, router, runner audio environment, or audio-server setup. The inspected session/lobby change was controller hotplug ordering, not a new audio architecture. [H5][H5]

Therefore adding a PulseAudio daemon to Wolf UI is not the appropriate compatibility change. It should continue connecting to the injected `PULSE_SERVER` and playing into `PULSE_SINK`. The targeted-sink experiment below confirms that the published Ubuntu UI honors that route in the tested setup. The user has now confirmed the affected Fedora UI tag; their Wolf revision and deployed UI digest remain unspecified.

## Published-image experiments

Both tested images identify this checkout's revision in their OCI labels. Tags were resolved on the investigation date:

| Image | Manifest digest | Runtime | Result |
| --- | --- | --- | --- |
| `ghcr.io/baz00k/wolf-ui-next:edge` | `sha256:601d12cc5032764a28843f73fffba3fc7a8f6e3af8cbed38a6642fe7cad80b46` | Ubuntu 25.04, WebKitGTK 2.50.4 | Assets decode; programmatic action dispatch and sound playback produce PulseAudio output |
| `ghcr.io/baz00k/wolf-ui-next:edge-fedora` | `sha256:c22aac84d49f656cffd89a63a5696da40b92010a044b2e4aaeb8327125a300d5` | Fedora 43, WebKitGTK 2.52.5 | Missing Good; output creation and all three asset decodes fail |

### Method and observations

- Started the repository's pinned Wolf image through `MISE_AUTO_INSTALL=false mise run dev`. Wolf and its embedded PulseAudio became healthy; the UI dev server then stopped because host WebKit development files are absent.
- Ran the **published production binaries inside Docker**, as root (matching `PUID=0`), against a headless Sway compositor and the pinned Wolf's PulseAudio socket. No alternate host runtime or application compatibility code was added.
- In the disposable containers only, instrumented the shipped JavaScript to report asset HTTP status, `decodeAudioData`, context state, dispatcher availability, and buffer-source starts. After buffers loaded, invoked `window.__wolfUiDispatchAction("right")` and then direct `window.__wolfUiSounds.play("select")`. These use the existing action/sound implementations; physical gamepad input was not exercised.
- All three hashed Ogg assets are present in both images and return HTTP 200 with `audio/ogg`. Normal startup in these runs loaded the sounds before focus navigation.
- Ubuntu successfully decoded navigation/select (0.110794 s, mono) and back (0.191088 s, stereo), automatically changed the context from suspended to running, and created a Pulse sink input. An 8-second float32 monitor recording contained 15,158 nonzero samples, peak 0.110657. No trusted keyboard/pointer gesture was supplied.
- Repeated Ubuntu with Wolf's bare-path form `PULSE_SERVER=/tmp/sockets/pulse-socket`, instead of `unix:/tmp/sockets/pulse-socket`: decoding and output still worked (17,298 nonzero recorded samples, peak 0.112457). The socket-prefix hypothesis was therefore **not reproduced in this tested environment**; subprocess sandbox behavior can differ elsewhere.
- With Ubuntu configured with the Wolf-shaped `PULSE_SINK=virtual_sink_audio_probe` and a matching test sink, WebKit created its sink input on that specific sink (index 1), not a global default. Its monitor recorded 14,664 nonzero samples (peak 0.029419). This verifies that the published app honors an injected PulseAudio target in the tested setup; the sink was created for diagnosis, not by a real Moonlight session.
- Fedora before package installation logged the errors below; all three `decodeAudioData` calls rejected with `EncodingError: Decoding failed`, leaving buffers unavailable and playback returning early.

```text
GStreamer element autoaudiosink not found. Please install it
Failed to create GStreamer audio sink element
GStreamer element deinterleave not found. Please install it
```

- Installed `dnf install -y --setopt=install_weak_deps=False gstreamer1-plugins-good` in that same disposable Fedora container. It installed Good plus four small dependencies; `gst-inspect-1.0` then found `autoaudiosink`, `deinterleave`, and `pulsesink`. Restarting the same binary decoded all three sounds, reached running state, and produced monitor audio (14,212 nonzero samples, peak 0.017029). No code, sound-gain, autoplay, or PulseAudio-server change was needed.

### Small reproducible dependency check

Run against the unmodified Fedora image (or the pinned digest above):

```bash
docker run --rm --entrypoint /bin/bash \
  ghcr.io/baz00k/wolf-ui-next:edge-fedora -lc '
    for element in autoaudiosink deinterleave pulsesink; do
      gst-inspect-1.0 "$element"
    done
  '
```

Each factory is missing. In a disposable copy, installing `gstreamer1-plugins-good` makes all three checks pass. This check is not by itself a streamed-audio test.

## Application findings and history

- UI sound assets/registration are present in `crates/wolf-ui/src/main.rs:34-37,86-90`; `ui-sounds.js` fetches and decodes them, then connects buffer sources through a gain node to the AudioContext destination.
- Sound support was added in `c4facb6` on **2026-06-16**. `f29de66` on **2026-08-21** lowered gains but did not remove playback. Fedora packaging was added in `3c890d3` on **2026-08-24**, including the weak-dependency-disabled install that omits Good.
- **Pointer/touch and Tab input have no equivalent sound path.** All calls to `uiSounds.play` are inside `__wolfUiDispatchAction` in `focus-navigation.js:447-499`. Rust dispatches keyboard actions/gamepad actions there (`input/provider.rs:245-250`), but normal pointer clicks and Tab focus movement do not. The back button's direct `navigator.go_back()` (`components/back_button.rs:24-27`) is another silent pointer path. This can explain silence when testing exclusively with a mouse/touch/Tab even on a healthy image.
- **An initialization-order bug is reproducible under fault injection:** `focus-navigation.js:24` captures `window.__wolfUiSounds` once, or a permanent no-op fallback. Delaying sound initialization by 250 ms in the disposable Ubuntu container caused focus navigation to capture the no-op; later decoding/context startup succeeded, but the normal action dispatcher stayed silent while direct `window.__wolfUiSounds.play("select")` worked. Normal unmodified startup runs did not demonstrate naturally reordered execution. This confirms the failure mode, not its frequency or that it caused the user's report.
- Decode failures log only a JavaScript warning; missing buffers return silently (`ui-sounds.js:26-28,53-63`), and resume errors are swallowed. There is no user-facing distinction between unavailable sounds and intentionally silent input.

## Recommended follow-up and limits

1. **Implemented:** explicitly add `gstreamer1-plugins-good` to the Fedora runtime package list in `Dockerfile`, retaining disabled weak dependencies.
2. **Implemented for Fedora:** build-time factory checks for `autoaudiosink`, `deinterleave`, `pulsesink`, `oggdemux`, and `vorbisdec`. A future automated WebAudio decode/output smoke test would strengthen regression coverage; Rust tests alone do not cover the system WebKit audio stack.
3. Fix sound initialization ownership/order and decide coherent select/back feedback for pointer/touch without double-playing keyboard/controller-generated clicks.
4. Confirm the reporting user's image tag/digest, input method, app-container `PULSE_SERVER`/`PULSE_SINK`, and session sink monitor if the problem is on Ubuntu. Successful shared-server playback does **not** prove the Moonlight session sink assignment or network/client output path.

No full Moonlight session was available. `/dev/uinput` and `/dev/uhid` are absent, so `mise run prod` prerequisites are not met. `mise run check` was attempted but cannot run because Cargo/Rust are not installed; `git diff --check` is the documentation validation. Test containers/compositor/diagnostic server were removed/stopped after investigation.

## Upstream primary-source research

The remaining sections explain the audio requirements and distinguish source facts from deployment-specific conditions.

Source snapshots: Wolf `stable` at `facb8e0ccd22fd6091cd904741e4dcf895e47b1b`; GoW at `bc4bb677e3376e17265317f23d887ecf42de8972`; WebKitGTK release tag `webkitgtk-2.50.0`. These are not assertions about the revisions inside the locally pinned Wolf image or floating `base-app:edge`. PulseAudio `master`, GStreamer docs, and distro package pages are moving references.

## Local context — verified by reading the repository

- At the investigated published revision, `Dockerfile` builds on `rust:trixie`, but **runtime** inherits `ghcr.io/games-on-whales/base-app:edge` (overridable). It installs WebKitGTK 4.1 using apt `--no-install-recommends` or dnf `install_weak_deps=False`; it does not explicitly list GStreamer plugins or configure PulseAudio. Builder distro/package presence does not establish runtime audio dependencies.
- `container-overlay/opt/gow/startup-app.sh` delegates to GoW's `launch-comp.sh` / `launcher`. Neither this script nor `usr/local/bin/webkit-helper-wrapper.sh` changes PulseAudio variables; the helper wrapper only forces Wayland. No local WebKit sandbox-disable setting appears in these files.
- `dev/compose.yml` shares `/tmp/sockets` between host and Wolf and sets `XDG_RUNTIME_DIR=/tmp/sockets`. The Wolf UI entry in `dev/wolf/config.toml` does not explicitly set PulseAudio variables or disable `start_audio_server`.
- `docs/development.md`, `mise.toml`, and `dev/scripts/{serve,prod}.sh` distinguish `mise run dev` (Wolf API plus `dx serve`, detached from Moonlight) from `mise run prod` (Wolf launches the actual UI container after a Moonlight session starts). API/dev success is not evidence that a child UI container's session audio route works.

## PulseAudio / Wolf — verified upstream requirements

1. **Wolf supplies the audio route; the app consumes it.** `start_audio_server` defaults to true. Wolf creates `virtual_sink_<session_id>` using `module-null-sink` and captures its `.monitor` with `pulsesrc`. Runner startup injects `PULSE_SERVER`, `PULSE_SINK=virtual_sink_<session_id>`, and `PULSE_SOURCE=virtual_sink_<session_id>.monitor`, and mounts the server socket into the app container. Thus the playback target is the session sink, not the monitor source. A separate app-local daemon or physical sound card is not part of this documented virtual-sink route. [W1][W1], [W2][W2], [W3][W3], [W4][W4]
2. **The shared runtime-directory mapping matters.** Wolf constructs the host socket path as `host_xdg_runtime_dir / basename(server_name)` and the app destination from the reported server name; its source explicitly acknowledges the assumed `<runtime-dir>/pulse-socket` layout. Custom external server layouts are not automatically portable through this mount construction. [W1][W1]
3. **Do not assume a PulseAudio sidecar is always required.** The inspected Wolf startup embeds PulseAudio when no external server is supplied, exports a **bare socket path**, and waits for the socket. C++ setup can fall back to `WolfPulseAudio` if connecting fails. Both the embedded command and GoW sidecar config use `module-native-protocol-unix auth-anonymous=1`. These configurations deliberately allow clients without a shared cookie; external servers may instead require authentication and accessible socket permissions. [W5][W5], [W6][W6], [W8][W8], [W9][W9], [P1][P1]
4. **Preserve the session defaults into the playback process.** PulseAudio's client configuration gives `PULSE_SERVER`, `PULSE_SINK`, and `PULSE_SOURCE` precedence over corresponding client.conf defaults. Its parser accepts both `/absolute/socket` and `unix:/absolute/socket`. `PULSE_COOKIE` selects a cookie file when authentication needs one; it is not an additional requirement for Wolf's anonymous socket configuration. [P1][P1], [P2][P2], [P3][P3], [P4][P4]

## WebKitGTK / GStreamer — verified upstream requirements

- **WebAudio needs a working GStreamer output pipeline, not just an enabled JavaScript API.** WebKitGTK 2.50.0 constructs a WebAudio source → `clocksync` → `audioconvert` → `audioresample` → audio sink pipeline. Its platform sink can fall back to `autoaudiosink`; failure to create or ready the sink makes output unavailable. [K1][K1], [K2][K2]
- **Good plugins are needed for output and WebAudio decoding.** GStreamer places `pulsesink` (PulseAudio output), `autoaudiosink` (sink discovery), and `deinterleave` (channel separation) in Good plugins. WebKit's GStreamer audio-file reader constructs `deinterleave`; having Ogg/Vorbis decoders alone is not sufficient for that decoding path. Synthesized audio still needs the output sink, but not compressed-media codecs. [G1][G1], [G2][G2], [G3][G3], [K11][K11]
- **Package names/dependency strength are distro-specific.** As a first-party packaging example, Debian trixie's `libwebkit2gtk-4.1-0` **depends** on `gstreamer1.0-plugins-base` and `gstreamer1.0-plugins-good`; libav/bad are **recommendations**. Good depends on `libpulse0`. Therefore `--no-install-recommends` does not by itself prove the PulseAudio sink is absent. This is not proof of the inherited Ubuntu/Fedora image contents. GoW's inspected apt base-app source additionally lists `pulseaudio-utils`; that supplies tools, not proof of WebKit's sink availability. [D1][D1], [D2][D2], [W7][W7]
- **Fedora has a specific weak-dependency trap.** Fedora's `webkit2gtk4.1` spec lists `gstreamer1-plugins-good` under **`Recommends`**, not `Requires`. Fedora documents that `--setopt=install_weak_deps=False` skips these weak dependencies; that is exactly the local Dockerfile's dnf setting. Fedora 43's Good package supplies `libgstautodetect.so`, `libgstpulseaudio.so`, and `libgstinterleave.so`. **The Fedora runtime must explicitly include `gstreamer1-plugins-good` (or otherwise deliberately provide its required elements); installing only WebKitGTK with weak dependencies disabled does not ensure working audio.** No evidence here requires the entire bad/libav plugin sets. [F1][F1], [F2][F2], [F3][F3], [G1][G1], [G2][G2], [G3][G3]
- **Docker socket visibility and WebKit subprocess visibility are separate requirements.** In WebKitGTK 2.50.0's bubblewrap launcher, `bindPulse()` binds the custom socket only when `PULSE_SERVER` begins with **`unix:`**. With it unset, it binds `<runtime-dir>/pulse`; with a bare custom path set, that branch does not bind the socket. It also exposes standard Pulse config locations and `/run/pulse`. Libpulse accepting a bare path does not imply the WebKit sandbox exposes that path. [K3][K3], [P3][P3]

## WebAudio activation / native gamepad eval — verified source facts

- The locked Wry 0.53.5 defaults `WebViewAttributes.autoplay` to true, enables WebAudio in GTK settings, and maps autoplay to WebKit `AutoplayPolicy::Allow`. Dioxus Desktop 0.7.9 constructs the Wry builder without overriding that default. This agrees with the no-gesture playback experiments; do not infer desktop behavior from Chrome autoplay rules. [R1][R1], [R2][R2], [R3][R3]

- WebKitGTK documents `enable-webaudio` default **true** and `media-playback-requires-user-gesture` default **false**. The 2.50.0 preference source also defaults audio-gesture restriction to false outside iOS; `Page::requiresUserGestureForAudioPlayback()` can override that through frame autoplay policy. Enabling WebAudio and authorizing playback are distinct, but mandatory gesture blocking is **not** a universal GTK default. [K4][K4], [K5][K5], [K6][K6], [K10][K10]
- If the restriction is active, `AudioContext` checks capture/transient activation (plus a site-quirk exception); a blocked `resume()` queues a reaction awaiting the running state rather than necessarily rejecting immediately. [K7][K7]
- **Native evaluation is not automatically gesture-less.** In 2.50.0, both `webkit_web_view_evaluate_javascript()` and legacy `webkit_web_view_run_javascript()` use the internal evaluator with `ForceUserGesture::Yes` and `RemoveTransientActivation::Yes`. `ScriptController` creates an activation-triggering gesture scope and consumes transient activation on its destruction. Inference: a native gamepad path calling these APIs can run synchronous audio-resume/start work with activation; work deferred outside that scope is not guaranteed the same activation. This does not verify which API or scheduling boundary the application uses. [K8][K8], [K9][K9]

## Exact primary-source links

[W1]: https://github.com/games-on-whales/wolf/blob/facb8e0ccd22fd6091cd904741e4dcf895e47b1b/src/moonlight-server/sessions/common.cpp#L19-L33
[W2]: https://github.com/games-on-whales/wolf/blob/facb8e0ccd22fd6091cd904741e4dcf895e47b1b/src/moonlight-server/state/configTOML.cpp#L199-L202
[W3]: https://github.com/games-on-whales/wolf/blob/facb8e0ccd22fd6091cd904741e4dcf895e47b1b/src/moonlight-server/sessions/moonlight.cpp#L142-L160
[W4]: https://github.com/games-on-whales/wolf/blob/facb8e0ccd22fd6091cd904741e4dcf895e47b1b/src/core/src/platforms/linux/pulseaudio/pulse.cpp#L152-L174
[W5]: https://github.com/games-on-whales/wolf/blob/facb8e0ccd22fd6091cd904741e4dcf895e47b1b/docker/startup.sh#L24-L47
[W6]: https://github.com/games-on-whales/wolf/blob/facb8e0ccd22fd6091cd904741e4dcf895e47b1b/docker/supervisord.conf#L25-L45
[W7]: https://github.com/games-on-whales/gow/blob/bc4bb677e3376e17265317f23d887ecf42de8972/images/base-app/build/Dockerfile#L145-L164
[W8]: https://github.com/games-on-whales/gow/blob/bc4bb677e3376e17265317f23d887ecf42de8972/images/pulseaudio/build/configs/default.pa
[W9]: https://github.com/games-on-whales/wolf/blob/facb8e0ccd22fd6091cd904741e4dcf895e47b1b/src/moonlight-server/wolf.cpp#L128-L169
[P1]: https://www.freedesktop.org/wiki/Software/PulseAudio/Documentation/User/Modules/#module-native-protocol-unix,tcp
[P2]: https://gitlab.freedesktop.org/pulseaudio/pulseaudio/-/blob/master/man/pulse-client.conf.5.xml.in
[P3]: https://gitlab.freedesktop.org/pulseaudio/pulseaudio/-/blob/master/src/pulsecore/parseaddr.c#L108-130
[P4]: https://gitlab.freedesktop.org/pulseaudio/pulseaudio/-/blob/master/src/pulse/client-conf.c
[K1]: https://github.com/WebKit/WebKit/blob/webkitgtk-2.50.0/Source/WebCore/platform/audio/gstreamer/AudioDestinationGStreamer.cpp#L124-L172
[K2]: https://github.com/WebKit/WebKit/blob/webkitgtk-2.50.0/Source/WebCore/platform/graphics/gstreamer/GStreamerCommon.cpp#L1027-L1038
[K3]: https://github.com/WebKit/WebKit/blob/webkitgtk-2.50.0/Source/WebKit/UIProcess/Launcher/glib/BubblewrapLauncher.cpp#L265-L301
[K4]: https://webkitgtk.org/reference/webkit2gtk/stable/property.Settings.enable-webaudio.html
[K5]: https://webkitgtk.org/reference/webkit2gtk/stable/property.Settings.media-playback-requires-user-gesture.html
[K6]: https://github.com/WebKit/WebKit/blob/webkitgtk-2.50.0/Source/WebCore/page/Page.cpp#L5878-L5883
[K7]: https://github.com/WebKit/WebKit/blob/webkitgtk-2.50.0/Source/WebCore/Modules/webaudio/AudioContext.cpp
[K8]: https://github.com/WebKit/WebKit/blob/webkitgtk-2.50.0/Source/WebKit/UIProcess/API/glib/WebKitWebView.cpp#L4286-L4302
[K9]: https://github.com/WebKit/WebKit/blob/webkitgtk-2.50.0/Source/WebCore/bindings/js/ScriptController.cpp#L619-L640
[K10]: https://github.com/WebKit/WebKit/blob/webkitgtk-2.50.0/Source/WTF/Scripts/Preferences/UnifiedWebPreferences.yaml#L6220-L6244
[G1]: https://gstreamer.freedesktop.org/documentation/pulseaudio/pulsesink.html
[G2]: https://gstreamer.freedesktop.org/documentation/autodetect/autoaudiosink.html
[G3]: https://gstreamer.freedesktop.org/documentation/interleave/deinterleave.html
[D1]: https://packages.debian.org/trixie/libwebkit2gtk-4.1-0
[D2]: https://packages.debian.org/trixie/gstreamer1.0-plugins-good
[K11]: https://github.com/WebKit/WebKit/blob/webkitgtk-2.50.0/Source/WebCore/platform/audio/gstreamer/AudioFileReaderGStreamer.cpp
[F1]: https://src.fedoraproject.org/rpms/webkitgtk/raw/f43/f/webkitgtk.spec
[F2]: https://docs.fedoraproject.org/en-US/packaging-guidelines/WeakDependencies/
[F3]: https://packages.fedoraproject.org/pkgs/gstreamer1-plugins-good/gstreamer1-plugins-good/fedora-43.html

[R1]: https://github.com/tauri-apps/wry/blob/wry-v0.53.5/src/lib.rs#L827
[R2]: https://github.com/tauri-apps/wry/blob/wry-v0.53.5/src/webkitgtk/mod.rs#L393-L400
[R3]: https://github.com/DioxusLabs/dioxus/blob/v0.7.9/packages/desktop/src/webview.rs#L359-L393

[H1]: https://github.com/games-on-whales/wolf/pull/422
[H2]: https://github.com/games-on-whales/wolf/pull/347
[H3]: https://github.com/games-on-whales/wolf/blob/facb8e0ccd22fd6091cd904741e4dcf895e47b1b/src/moonlight-server/audio/pulse_router.cpp#L212-L262
[H4]: https://github.com/games-on-whales/wolf/blob/facb8e0ccd22fd6091cd904741e4dcf895e47b1b/src/moonlight-server/sessions/lobbies.cpp#L133-L179
[H5]: https://github.com/games-on-whales/wolf/compare/7f416ec0f26bb4d95218af806cd1181fe5434598...facb8e0ccd22fd6091cd904741e4dcf895e47b1b
