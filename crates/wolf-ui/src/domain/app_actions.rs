use dioxus::prelude::*;
use wolf_api::ApiError;
use wolf_api::types::{
    RflReflectorWolfCoreEventsAppReflTypeRunner, RflReflectorWolfCoreEventsLobbyReflType,
    WolfApiCreateLobbyRequest, WolfApiCreateLobbyRequestRunner, WolfApiPartialClientSettings,
    WolfConfigAppCMDTagged, WolfCoreEventsAudioSettings, WolfCoreEventsVideoSettings,
};

use crate::api::ApiContext;
use crate::components::{AppAction, AppCardData};
use crate::domain::apps::{docker_image, is_app_lobby};
use crate::domain::session::current_session_id;

pub(crate) async fn run_app_action(
    profile_id: String,
    app: AppCardData,
    action: AppAction,
    pin: Option<Vec<i64>>,
    on_progress: impl FnMut(f64),
) -> Result<bool, String> {
    match action {
        AppAction::Start => start_app(profile_id, &app, false, None).await,
        AppAction::Connect => connect_app(&profile_id, &app).await,
        AppAction::StartCoop => start_app(profile_id, &app, true, pin).await,
        AppAction::Stop => stop_app(&profile_id, &app).await,
        AppAction::CheckUpdate => pull_image(&app, on_progress).await,
        AppAction::Download => pull_image(&app, on_progress).await,
    }
}

async fn start_app(
    profile_id: String,
    app: &AppCardData,
    multi_user: bool,
    pin: Option<Vec<i64>>,
) -> Result<bool, String> {
    let api = ApiContext::consume();
    let session_id = current_session_id()?;
    let session = api
        .sessions()
        .by_client_id(&session_id)
        .await
        .map_err(api_error_message)?
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
            let lobby_name = if multi_user {
                creator_lobby_name(&api, &profile_id)
                    .await
                    .unwrap_or_else(|_| app.title.clone())
            } else {
                app.title.clone()
            };
            let lobby = build_lobby(
                profile_id,
                app,
                &session,
                multi_user,
                lobby_name,
                pin.clone(),
            );
            api.lobbies()
                .create(&lobby)
                .await
                .map_err(api_error_message)?
                .lobby_id
        }
    };

    api.lobbies()
        .join(lobby_id, session_id, pin)
        .await
        .map_err(api_error_message)?;
    Ok(true)
}

pub(crate) async fn join_lobby(
    lobby_id: String,
    pin: Option<Vec<i64>>,
) -> Result<bool, std::io::Error> {
    let api = ApiContext::consume();
    let session_id = current_session_id().map_err(std::io::Error::other)?;

    api.lobbies()
        .join(lobby_id, session_id, pin)
        .await
        .map_err(|error| std::io::Error::other(api_error_message(error)))?;
    Ok(true)
}

pub(crate) async fn stop_lobby(
    lobby_id: String,
    pin: Option<Vec<i64>>,
) -> Result<bool, std::io::Error> {
    ApiContext::consume()
        .lobbies()
        .stop(lobby_id, pin)
        .await
        .map_err(|error| std::io::Error::other(api_error_message(error)))?;
    Ok(true)
}

fn api_error_message(error: ApiError) -> String {
    match error {
        ApiError::Timeout => "Request timed out".to_string(),
        ApiError::Wolf { message, .. } => message,
        ApiError::Status { body, .. } => body,
        error => error.to_string(),
    }
}

async fn pull_image(app: &AppCardData, on_progress: impl FnMut(f64)) -> Result<bool, String> {
    let image =
        docker_image(&app.source).ok_or_else(|| "This app has no Docker image.".to_string())?;

    let api = ApiContext::consume();
    let downloaded = api
        .docker()
        .pull_image(&image, on_progress)
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
    Ok(downloaded)
}

async fn connect_app(profile_id: &str, app: &AppCardData) -> Result<bool, String> {
    let api = ApiContext::consume();
    let session_id = current_session_id()?;
    let lobby = running_lobby(&api, profile_id, app, false)
        .await?
        .ok_or_else(|| "No running lobby found for this app.".to_string())?;
    api.lobbies()
        .join(lobby.id, session_id, None)
        .await
        .map_err(|error| error.to_string())?;
    Ok(true)
}

async fn stop_app(profile_id: &str, app: &AppCardData) -> Result<bool, String> {
    let api = ApiContext::consume();
    let lobby = running_lobby(&api, profile_id, app, false)
        .await?
        .ok_or_else(|| "No running lobby found for this app.".to_string())?;
    api.lobbies()
        .stop(lobby.id, None)
        .await
        .map_err(|error| error.to_string())?;
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
    name: String,
    pin: Option<Vec<i64>>,
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
        name,
        pin,
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

async fn creator_lobby_name(api: &wolf_api::WolfApi, profile_id: &str) -> Result<String, String> {
    let profile = api
        .profiles()
        .list()
        .await
        .map_err(|error| error.to_string())?
        .profiles
        .into_iter()
        .find(|profile| profile.id == profile_id)
        .ok_or_else(|| "Profile not found.".to_string())?;

    Ok(format!("{}'s lobby", profile.name))
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
