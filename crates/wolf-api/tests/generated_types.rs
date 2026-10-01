use serde_json::{Value, json};
use wolf_api::types::{
    App, AppCmd, AppCmdType, AppDocker, AppDockerType, GenericErrorResponse, Runner,
    StopLobbyEvent, UpdateClientSettingsRequest,
};

#[test]
fn runner_variants_round_trip_with_wire_discriminators() {
    let cases = [
        (
            Runner::Cmd(AppCmd {
                run_cmd: "game".to_string(),
                type_: AppCmdType::Process,
            }),
            "process",
        ),
        (
            Runner::Docker(AppDocker {
                base_create_json: None,
                devices: Vec::new(),
                env: Vec::new(),
                image: "example/game:latest".to_string(),
                mounts: Vec::new(),
                name: "game".to_string(),
                ports: Vec::new(),
                type_: AppDockerType::Docker,
            }),
            "docker",
        ),
    ];

    for (runner, expected_type) in cases {
        let value = serde_json::to_value(&runner).expect("runner should serialize");
        assert_eq!(value["type"], expected_type);
        assert_eq!(
            serde_json::from_value::<Runner>(value).expect("runner should deserialize"),
            runner
        );
    }
}

#[test]
fn app_accepts_absent_or_null_nullable_icon() {
    let mut value = serde_json::to_value(App {
        av1_gst_pipeline: String::new(),
        h264_gst_pipeline: String::new(),
        hevc_gst_pipeline: String::new(),
        icon_png_path: None,
        id: "game".to_string(),
        opus_gst_pipeline: String::new(),
        render_node: String::new(),
        runner: Runner::Cmd(AppCmd {
            run_cmd: "game".to_string(),
            type_: AppCmdType::Process,
        }),
        start_audio_server: false,
        start_virtual_compositor: false,
        support_hdr: false,
        title: "Game".to_string(),
    })
    .expect("app should serialize");
    assert!(value.get("icon_png_path").is_none());
    let app = serde_json::from_value::<App>(value.clone()).expect("absent icon should deserialize");
    assert_eq!(app.icon_png_path, None);

    value
        .as_object_mut()
        .expect("app should serialize as an object")
        .insert("icon_png_path".to_string(), Value::Null);

    let app = serde_json::from_value::<App>(value).expect("null icon should deserialize");

    assert_eq!(app.icon_png_path, None);
}

#[test]
fn required_nullable_client_settings_fields_must_be_present() {
    let value = json!({
        "client_id": "client",
        "app_state_folder": null,
        "settings": null,
    });
    let request = serde_json::from_value::<UpdateClientSettingsRequest>(value.clone())
        .expect("required nullable fields should accept null");

    assert_eq!(request.app_state_folder, None);
    assert_eq!(request.settings, None);
    assert_eq!(
        serde_json::to_value(request).expect("client settings request should serialize"),
        value
    );

    for field in ["app_state_folder", "settings"] {
        let mut missing_field = value.clone();
        missing_field
            .as_object_mut()
            .expect("request should be an object")
            .remove(field);
        let error = serde_json::from_value::<UpdateClientSettingsRequest>(missing_field)
            .expect_err("missing required nullable field should be rejected");

        assert!(
            error
                .to_string()
                .contains(&format!("missing field `{field}`"))
        );
    }
}

#[test]
fn absent_stop_lobby_pin_is_omitted() {
    let value = serde_json::to_value(StopLobbyEvent {
        lobby_id: "lobby".to_string(),
        pin: None,
    })
    .expect("stop lobby event should serialize");

    assert_eq!(value, json!({"lobby_id": "lobby"}));
}

#[test]
fn generic_error_response_matches_wolf_envelope() {
    let value = json!({"success": false, "error": "failed"});
    let response =
        serde_json::from_value::<GenericErrorResponse>(value.clone()).expect("error should parse");

    assert_eq!(response.error, "failed");
    assert!(!response.success);
    assert_eq!(
        serde_json::to_value(response).expect("error should serialize"),
        value
    );
}
