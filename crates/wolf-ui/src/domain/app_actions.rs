use std::collections::HashMap;

use dioxus::prelude::*;
use wolf_api::types::{
    RflReflectorWolfCoreEventsAppReflTypeRunner, RflReflectorWolfCoreEventsLobbyReflType,
    WolfApiCreateLobbyRequest, WolfApiCreateLobbyRequestRunner, WolfApiPartialClientSettings,
    WolfConfigAppCMDTagged, WolfCoreEventsAudioSettings, WolfCoreEventsVideoSettings,
};

use crate::api::ApiContext;
use crate::components::{AppAction, AppCardData};
use crate::domain::apps::{docker_image, is_app_lobby};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActionStatusKind {
    Loading,
    Progress,
    Success,
    Error,
}

#[derive(Clone, PartialEq)]
pub(crate) struct ActionStatus {
    pub kind: ActionStatusKind,
    pub app_id: String,
    pub message: String,
    pub progress: Option<f64>,
}

pub(crate) type ActionStatuses = HashMap<String, ActionStatus>;

pub(crate) async fn run_app_action(
    profile_id: String,
    app: AppCardData,
    action: AppAction,
    statuses: Signal<ActionStatuses>,
) -> Result<bool, String> {
    match action {
        AppAction::Start => start_app(profile_id, &app, false, statuses).await,
        AppAction::Connect => connect_app(&profile_id, &app, statuses).await,
        AppAction::StartCoop => start_app(profile_id, &app, true, statuses).await,
        AppAction::Stop => stop_app(&profile_id, &app, statuses).await,
        AppAction::CheckUpdate => pull_image(&app, true, statuses).await,
        AppAction::Download => pull_image(&app, false, statuses).await,
    }
}

async fn start_app(
    profile_id: String,
    app: &AppCardData,
    multi_user: bool,
    mut statuses: Signal<ActionStatuses>,
) -> Result<bool, String> {
    set_status(
        &mut statuses,
        action_status(
            ActionStatusKind::Loading,
            &app.id,
            if multi_user {
                "Creating co-op lobby..."
            } else {
                "Starting lobby..."
            },
            None,
        ),
    );

    let api = ApiContext::consume();
    let session_id = session_id()?;
    let session = api
        .sessions()
        .by_client_id(&session_id)
        .await
        .map_err(|error| error.to_string())?
        .unwrap_or_else(fallback_session);
    let lobby_id = if multi_user {
        None
    } else {
        running_lobby(&api, &profile_id, app, false)
            .await?
            .map(|lobby| lobby.id)
    };
    let lobby_id = match lobby_id {
        Some(lobby_id) => lobby_id,
        None => {
            let lobby = build_lobby(profile_id, app, &session, multi_user);
            api.lobbies()
                .create(&lobby)
                .await
                .map_err(|error| error.to_string())?
                .lobby_id
        }
    };

    set_status(
        &mut statuses,
        action_status(ActionStatusKind::Loading, &app.id, "Joining lobby...", None),
    );
    api.lobbies()
        .join(lobby_id, session_id, None)
        .await
        .map_err(|error| error.to_string())?;

    set_status(
        &mut statuses,
        action_status(ActionStatusKind::Success, &app.id, "Lobby joined.", None),
    );
    Ok(true)
}

async fn pull_image(
    app: &AppCardData,
    check_update: bool,
    mut statuses: Signal<ActionStatuses>,
) -> Result<bool, String> {
    let image =
        docker_image(&app.source).ok_or_else(|| "This app has no Docker image.".to_string())?;
    let label = if check_update {
        "Checking for image updates..."
    } else {
        "Downloading image..."
    };
    set_status(
        &mut statuses,
        action_status(ActionStatusKind::Progress, &app.id, label, Some(0.0)),
    );

    let progress_app_id = app.id.clone();
    let api = ApiContext::consume();
    let downloaded = api
        .docker()
        .pull_image(&image, move |progress| {
            set_status(
                &mut statuses,
                action_status(
                    ActionStatusKind::Progress,
                    &progress_app_id,
                    if progress >= 100.0 {
                        "Finalizing image..."
                    } else {
                        label
                    },
                    Some(progress),
                ),
            );
        })
        .await
        .map_err(|error| error.to_string())?;

    let exists = api
        .docker()
        .image_exists(&image)
        .await
        .map_err(|error| error.to_string())?;
    if !exists {
        return Err("Wolf reported pull success, but the image is still missing.".to_string());
    }

    set_status(
        &mut statuses,
        action_status(
            ActionStatusKind::Success,
            &app.id,
            if downloaded {
                "Image updated."
            } else {
                "Image already up to date."
            },
            Some(100.0),
        ),
    );
    Ok(downloaded)
}

async fn connect_app(
    profile_id: &str,
    app: &AppCardData,
    mut statuses: Signal<ActionStatuses>,
) -> Result<bool, String> {
    set_status(
        &mut statuses,
        action_status(ActionStatusKind::Loading, &app.id, "Connecting...", None),
    );

    let api = ApiContext::consume();
    let session_id = session_id()?;
    let lobby = running_lobby(&api, profile_id, app, false)
        .await?
        .ok_or_else(|| "No running lobby found for this app.".to_string())?;
    api.lobbies()
        .join(lobby.id, session_id, None)
        .await
        .map_err(|error| error.to_string())?;

    set_status(
        &mut statuses,
        action_status(ActionStatusKind::Success, &app.id, "Connected.", None),
    );
    Ok(true)
}

async fn stop_app(
    profile_id: &str,
    app: &AppCardData,
    mut statuses: Signal<ActionStatuses>,
) -> Result<bool, String> {
    set_status(
        &mut statuses,
        action_status(ActionStatusKind::Loading, &app.id, "Stopping...", None),
    );

    let api = ApiContext::consume();
    let lobby = running_lobby(&api, profile_id, app, false)
        .await?
        .ok_or_else(|| "No running lobby found for this app.".to_string())?;
    api.lobbies()
        .stop(lobby.id, None)
        .await
        .map_err(|error| error.to_string())?;

    set_status(
        &mut statuses,
        action_status(ActionStatusKind::Success, &app.id, "Stopped.", None),
    );
    Ok(true)
}

async fn running_lobby(
    api: &wolf_api::WolfApi,
    profile_id: &str,
    app: &AppCardData,
    multi_user: bool,
) -> Result<Option<RflReflectorWolfCoreEventsLobbyReflType>, String> {
    let lobbies = api
        .lobbies()
        .list()
        .await
        .map_err(|error| error.to_string())?;
    Ok(lobbies
        .lobbies
        .into_iter()
        .find(|lobby| is_app_lobby(profile_id, &app.source, lobby, multi_user)))
}

fn build_lobby(
    profile_id: String,
    app: &AppCardData,
    session: &wolf_api::sessions::Session,
    multi_user: bool,
) -> WolfApiCreateLobbyRequest {
    let runner_name = match &app.source.runner {
        RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppDockerTagged(runner) => {
            &runner.name
        }
        RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppCMDTagged(_) => &app.source.id,
    };

    WolfApiCreateLobbyRequest {
        audio_settings: WolfCoreEventsAudioSettings {
            channel_count: session.audio_channel_count,
        },
        client_settings: session
            .client_settings
            .as_ref()
            .map(partial_client_settings),
        icon_png_path: app.source.icon_png_path.clone(),
        multi_user,
        name: app.title.clone(),
        pin: None,
        profile_id: profile_id.clone(),
        runner: create_lobby_runner(&app.source.runner),
        runner_state_folder: format!("profile-data/{profile_id}/{runner_name}"),
        stop_when_everyone_leaves: false,
        video_settings: WolfCoreEventsVideoSettings {
            height: session.video_height,
            refresh_rate: session.video_refresh_rate,
            runner_render_node: app.source.render_node.clone(),
            video_producer_buffer_caps: std::env::var("WOLF_VIDEO_BUFFER_CAPS").unwrap_or_default(),
            wayland_render_node: app.source.render_node.clone(),
            width: session.video_width,
        },
    }
}

fn create_lobby_runner(
    runner: &RflReflectorWolfCoreEventsAppReflTypeRunner,
) -> WolfApiCreateLobbyRequestRunner {
    match runner {
        RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppDockerTagged(runner) => {
            WolfApiCreateLobbyRequestRunner::WolfConfigAppDockerTagged(runner.clone())
        }
        RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppCMDTagged(runner) => {
            WolfApiCreateLobbyRequestRunner::WolfConfigAppCMDTagged(WolfConfigAppCMDTagged {
                run_cmd: runner.run_cmd.clone(),
            })
        }
    }
}

fn partial_client_settings(
    settings: &wolf_api::types::WolfConfigClientSettings,
) -> WolfApiPartialClientSettings {
    WolfApiPartialClientSettings {
        controllers_override: Some(settings.controllers_override.clone()),
        h_scroll_acceleration: Some(settings.h_scroll_acceleration),
        mouse_acceleration: Some(settings.mouse_acceleration),
        run_gid: Some(settings.run_gid),
        run_uid: Some(settings.run_uid),
        v_scroll_acceleration: Some(settings.v_scroll_acceleration),
    }
}

fn session_id() -> Result<String, String> {
    std::env::var("WOLF_SESSION_ID")
        .ok()
        .filter(|session_id| !session_id.trim().is_empty())
        .ok_or_else(|| "WOLF_SESSION_ID is not set.".to_string())
}

fn fallback_session() -> wolf_api::sessions::Session {
    wolf_api::sessions::Session {
        aes_iv: String::new(),
        aes_key: String::new(),
        app_id: None,
        audio_channel_count: 2,
        client_id: None,
        client_ip: String::new(),
        client_settings: None,
        rtsp_fake_ip: String::new(),
        video_height: 1080,
        video_refresh_rate: 60,
        video_width: 1920,
    }
}

pub(crate) fn error_status(app_id: String, message: String) -> ActionStatus {
    ActionStatus {
        kind: ActionStatusKind::Error,
        app_id,
        message,
        progress: None,
    }
}

fn action_status(
    kind: ActionStatusKind,
    app_id: &str,
    message: &str,
    progress: Option<f64>,
) -> ActionStatus {
    ActionStatus {
        kind,
        app_id: app_id.to_string(),
        message: message.to_string(),
        progress,
    }
}

fn set_status(statuses: &mut Signal<ActionStatuses>, status: ActionStatus) {
    statuses.write().insert(status.app_id.clone(), status);
}
