use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use wolf_api::{ApiTransport, ClientConfig, WolfApi, client_with_config};

use super::{
    ClientSettings, ClientSettingsMotionControllerOverride,
    PartialClientSettingsMotionControllerOverride, partial_client_settings, start_app,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

struct Exchange {
    method: &'static str,
    path: &'static str,
    body: Value,
    response: Value,
    status: u16,
}

impl Exchange {
    fn get(path: &'static str, response: Value) -> Self {
        Self {
            method: "GET",
            path,
            body: Value::Null,
            response,
            status: 200,
        }
    }

    fn post(path: &'static str, body: Value, response: Value) -> Self {
        Self {
            method: "POST",
            path,
            body,
            response,
            status: 200,
        }
    }

    fn failed(mut self) -> Self {
        self.status = 400;
        self.response = json!({"success": false, "error": "Wolf refused the request"});
        self
    }
}

/// Exercises the real Wolf client, including serialization and response decoding.
/// Each exchange checks order and body; a timeout bounds missing-request failures.
fn launch(
    exchanges: Vec<Exchange>,
    app: Value,
    coop: bool,
    pin: Option<Vec<i64>>,
) -> Result<bool, String> {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        for exchange in exchanges {
            let deadline = Instant::now() + REQUEST_TIMEOUT;
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            Instant::now() < deadline,
                            "missing {} {}",
                            exchange.method,
                            exchange.path
                        );
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("accept request: {error}"),
                }
            };
            stream.set_read_timeout(Some(REQUEST_TIMEOUT)).unwrap();
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert_eq!(
                line.trim(),
                format!("{} {} HTTP/1.1", exchange.method, exchange.path)
            );
            let mut length = 0;
            loop {
                line.clear();
                assert_ne!(reader.read_line(&mut line).unwrap(), 0);
                if line == "\r\n" {
                    break;
                }
                if let Some((name, value)) = line.split_once(':')
                    && name.eq_ignore_ascii_case("content-length")
                {
                    length = value.trim().parse().unwrap();
                }
            }
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            let body = if body.is_empty() {
                Value::Null
            } else {
                serde_json::from_slice(&body).unwrap()
            };
            assert_eq!(body, exchange.body, "{} body", exchange.path);
            let response = exchange.response.to_string();
            write!(
                stream,
                "HTTP/1.1 {} Test\r\n\
                 Content-Type: application/json\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\r\n{}",
                exchange.status,
                response.len(),
                response
            )
            .unwrap();
        }
        listener
    });
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let api: WolfApi = client_with_config(
                ClientConfig::new()
                    .transport(ApiTransport::Tcp)
                    .base_url(url)
                    .max_retries(0)
                    .connect_timeout(REQUEST_TIMEOUT)
                    .request_timeout(REQUEST_TIMEOUT),
            )
            .unwrap();
            start_app(
                &api,
                "profile-1".into(),
                &serde_json::from_value(app).unwrap(),
                "session-1".into(),
                "video/x-raw".into(),
                coop,
                pin,
            )
            .await
        });
    let listener = server.join().unwrap();
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
        "unexpected request after the launch sequence"
    );
    result
}

fn app() -> Value {
    json!({
        "id": "app-1", "title": "Game", "render_node": "/dev/dri/renderD128",
        "icon_png_path": "/icons/game.png", "runner": {"type": "process", "run_cmd": "play"},
        "av1_gst_pipeline": "", "h264_gst_pipeline": "", "hevc_gst_pipeline": "", "opus_gst_pipeline": "",
        "start_audio_server": false, "start_virtual_compositor": true, "support_hdr": false
    })
}

fn sessions() -> Exchange {
    Exchange::get("/api/v1/sessions", json!({"success": true, "sessions": []}))
}

fn lobbies(items: Value) -> Exchange {
    Exchange::get(
        "/api/v1/lobbies",
        json!({"success": true, "lobbies": items}),
    )
}

fn lobby(profile: &str, coop: bool) -> Value {
    json!({"id": "existing", "name": "Game", "started_by_profile_id": profile,
        "multi_user": coop, "pin_required": false, "connected_sessions": [],
        "runner": app()["runner"], "stop_when_everyone_leaves": false})
}

fn create_body() -> Value {
    json!({
        "audio_settings": {"channel_count": 2}, "icon_png_path": "/icons/game.png",
        "client_settings": null, "pin": null,
        "multi_user": false, "name": "Game", "profile_id": "profile-1",
        "runner": app()["runner"], "runner_state_folder": "profile-data/profile-1/app-1",
        "stop_when_everyone_leaves": false,
        "video_settings": {"height": 1080, "width": 1920, "refresh_rate": 60,
            "runner_render_node": "/dev/dri/renderD128", "wayland_render_node": "/dev/dri/renderD128",
            "video_producer_buffer_caps": "video/x-raw"}
    })
}

fn create(body: Value) -> Exchange {
    Exchange::post(
        "/api/v1/lobbies/create",
        body,
        json!({"success": true, "lobby_id": "created"}),
    )
}

fn join(id: &str, pin: Option<Vec<i64>>) -> Exchange {
    let mut body = json!({"lobby_id": id, "moonlight_session_id": "session-1"});
    if let Some(pin) = pin {
        body["pin"] = json!(pin);
    }
    Exchange::post("/api/v1/lobbies/join", body, json!({"success": true}))
}

#[test]
fn ordinary_start_reuses_only_a_matching_private_lobby() {
    let mut other_profile = lobby("other-profile", false);
    other_profile["id"] = json!("wrong-profile");
    let mut coop = lobby("profile-1", true);
    coop["id"] = json!("wrong-mode");
    let mut other_app = lobby("profile-1", false);
    other_app["id"] = json!("wrong-app");
    other_app["name"] = json!("Another game");
    let exchanges = vec![
        sessions(),
        lobbies(json!([
            other_profile,
            coop,
            other_app,
            lobby("profile-1", false)
        ])),
        join("existing", None),
    ];
    assert_eq!(launch(exchanges, app(), false, None), Ok(true));
}

#[test]
fn docker_start_reuses_runner_name_even_when_lobby_title_differs() {
    let mut app = app();
    app["runner"] = json!({"type": "docker", "name": "game-runner", "image": "game:latest",
        "devices": [], "env": [], "mounts": [], "ports": []});
    let mut existing = lobby("profile-1", false);
    existing["name"] = json!("Old game title");
    existing["runner"] = app["runner"].clone();

    assert_eq!(
        launch(
            vec![
                sessions(),
                lobbies(json!([existing])),
                join("existing", None)
            ],
            app,
            false,
            None,
        ),
        Ok(true)
    );
}

#[test]
fn missing_session_uses_fallback_stream_settings_then_creates_and_joins() {
    assert_eq!(
        launch(
            vec![
                sessions(),
                lobbies(json!([])),
                create(create_body()),
                join("created", None)
            ],
            app(),
            false,
            None
        ),
        Ok(true)
    );
}

#[test]
fn create_preserves_session_settings_and_docker_state_folder() {
    let settings = json!({"controllers_override": ["XBOX", "PS", "NINTENDO", "AUTO"],
        "motion_controller_override": "PS", "h_scroll_acceleration": 1.5,
        "mouse_acceleration": 2.0, "v_scroll_acceleration": 3.0, "run_uid": 1001, "run_gid": 1002});
    let sessions = Exchange::get(
        "/api/v1/sessions",
        json!({"success": true, "sessions": [
            {"client_id": "other", "aes_iv": "", "aes_key": "", "client_ip": "", "rtsp_fake_ip": "",
                "audio_channel_count": 2, "video_width": 640, "video_height": 480, "video_refresh_rate": 30},
            {"client_id": "session-1", "aes_iv": "", "aes_key": "", "client_ip": "", "rtsp_fake_ip": "",
                "audio_channel_count": 6, "video_width": 2560, "video_height": 1440, "video_refresh_rate": 120,
                "client_settings": settings}
        ]}),
    );
    let mut app = app();
    app["runner"] = json!({"type": "docker", "name": "game-runner", "image": "game:latest",
        "devices": [], "env": [], "mounts": [], "ports": []});
    let mut body = create_body();
    body["runner"] = app["runner"].clone();
    body["runner_state_folder"] = json!("profile-data/profile-1/game-runner");
    body["client_settings"] = settings;
    body["audio_settings"]["channel_count"] = json!(6);
    body["video_settings"]["width"] = json!(2560);
    body["video_settings"]["height"] = json!(1440);
    body["video_settings"]["refresh_rate"] = json!(120);
    assert_eq!(
        launch(
            vec![
                sessions,
                lobbies(json!([])),
                create(body),
                join("created", None)
            ],
            app,
            false,
            None
        ),
        Ok(true)
    );
}

#[test]
fn coop_start_skips_reuse_and_passes_pin_to_create_and_join() {
    let profiles = Exchange::get(
        "/api/v1/profiles",
        json!({"success": true, "profiles": [
            {"id": "profile-1", "name": "Player", "apps": [], "icon_png_path": ""}
        ]}),
    );
    let mut body = create_body();
    body["multi_user"] = json!(true);
    body["name"] = json!("Player's lobby");
    body["pin"] = json!([1, 2, 3, 4]);
    assert_eq!(
        launch(
            vec![
                sessions(),
                profiles,
                create(body),
                join("created", Some(vec![1, 2, 3, 4]))
            ],
            app(),
            true,
            Some(vec![1, 2, 3, 4])
        ),
        Ok(true)
    );
}

#[test]
fn coop_profile_failure_falls_back_to_app_title() {
    let profiles = Exchange::get("/api/v1/profiles", json!({})).failed();
    let mut body = create_body();
    body["multi_user"] = json!(true);
    assert_eq!(
        launch(
            vec![sessions(), profiles, create(body), join("created", None)],
            app(),
            true,
            None
        ),
        Ok(true)
    );
}

#[test]
fn session_lookup_failure_does_not_create_or_join() {
    assert_eq!(
        launch(vec![sessions().failed()], app(), false, None),
        Err("Wolf refused the request".into())
    );
}

#[test]
fn lobby_lookup_failure_does_not_create_or_join() {
    assert!(
        launch(
            vec![sessions(), lobbies(json!([])).failed()],
            app(),
            false,
            None
        )
        .unwrap_err()
        .contains("Wolf refused the request")
    );
}

#[test]
fn create_failure_does_not_join() {
    assert_eq!(
        launch(
            vec![
                sessions(),
                lobbies(json!([])),
                create(create_body()).failed()
            ],
            app(),
            false,
            None
        ),
        Err("Wolf refused the request".into())
    );
}

#[test]
fn join_failure_does_not_stop_created_lobby() {
    assert_eq!(
        launch(
            vec![
                sessions(),
                lobbies(json!([])),
                create(create_body()),
                join("created", None).failed()
            ],
            app(),
            false,
            None
        ),
        Err("Wolf refused the request".into())
    );
}

#[test]
fn partial_client_settings_preserve_motion_controller_override() {
    let cases = [
        (
            ClientSettingsMotionControllerOverride::Xbox,
            PartialClientSettingsMotionControllerOverride::Xbox,
        ),
        (
            ClientSettingsMotionControllerOverride::Ps,
            PartialClientSettingsMotionControllerOverride::Ps,
        ),
        (
            ClientSettingsMotionControllerOverride::Nintendo,
            PartialClientSettingsMotionControllerOverride::Nintendo,
        ),
        (
            ClientSettingsMotionControllerOverride::Auto,
            PartialClientSettingsMotionControllerOverride::Auto,
        ),
    ];

    for (source, expected) in cases {
        let settings = ClientSettings {
            controllers_override: Vec::new(),
            h_scroll_acceleration: 1.0,
            motion_controller_override: source,
            mouse_acceleration: 2.0,
            run_gid: 1000,
            run_uid: 1000,
            v_scroll_acceleration: 3.0,
        };

        assert_eq!(
            partial_client_settings(&settings).motion_controller_override,
            Some(expected)
        );
    }
}
