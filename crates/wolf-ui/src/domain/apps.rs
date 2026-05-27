use std::collections::HashMap;

use dioxus::prelude::*;
use futures_util::StreamExt;
use wolf_api::lobbies::Lobby;
use wolf_api::profiles::App;
use wolf_api::types::{RflReflectorWolfCoreEventsAppReflTypeRunner, WolfConfigAppDockerTagged};

use crate::api::ApiContext;
use crate::components::{AppAction, AppCardData, AppStatus, AppStatusKind, AppStatusTone};
use crate::domain::image_loader::load_image_src;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppFilter {
    Default,
    Available,
    Alphabetical,
}

#[derive(Clone, PartialEq)]
pub(crate) struct AppsState {
    pub apps: Vec<App>,
    pub lobbies: Vec<Lobby>,
    pub images: HashMap<String, bool>,
    pub covers: HashMap<String, String>,
}

pub(crate) async fn load_apps_state(profile_id: String) -> Result<AppsState, String> {
    let api = ApiContext::consume();
    let response = api
        .profiles()
        .apps(&profile_id)
        .await
        .map_err(|error| error.to_string())?;
    let lobby_response = api
        .lobbies()
        .list()
        .await
        .map_err(|error| error.to_string())?;
    let images = image_availability(&api, &response.apps).await;
    let covers = cover_images(&api, &response.apps).await;

    Ok(AppsState {
        apps: response.apps,
        lobbies: lobby_response.lobbies,
        images,
        covers,
    })
}

pub(crate) async fn listen_for_lobby_events(profile_id: String, lobbies: Signal<Vec<Lobby>>) {
    let Ok(response) = ApiContext::consume().events().connect().await else {
        return;
    };
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut event_type = String::new();
    let mut data = String::new();

    while let Some(Ok(chunk)) = stream.next().await {
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(index) = buffer.find('\n') {
            let line = buffer[..index].trim_end_matches('\r').to_string();
            buffer.replace_range(..=index, "");

            if line.is_empty() {
                apply_sse_event(&profile_id, &event_type, &data, lobbies);
                event_type.clear();
                data.clear();
            } else if let Some(value) = line.strip_prefix("event: ") {
                event_type = value.to_string();
            } else if let Some(value) = line.strip_prefix("data: ") {
                data.push_str(value);
            }
        }
    }
}

pub(crate) fn sorted_apps(
    profile_id: &str,
    apps: Vec<App>,
    lobbies: &[Lobby],
    images: &HashMap<String, bool>,
    covers: &HashMap<String, String>,
    filter: AppFilter,
) -> Vec<AppCardData> {
    let mut apps = apps
        .into_iter()
        .map(|app| app_card_data(profile_id, app, lobbies, images, covers))
        .collect::<Vec<_>>();

    match filter {
        AppFilter::Default => {}
        AppFilter::Available => {
            apps.sort_by_key(|app| !matches!(app.status.tone, AppStatusTone::Ready));
        }
        AppFilter::Alphabetical => apps.sort_by_key(|app| app.title.to_lowercase()),
    }

    apps
}

pub(crate) fn selected_app_data(
    profile_id: &str,
    apps: &[App],
    lobbies: &[Lobby],
    images: &HashMap<String, bool>,
    covers: &HashMap<String, String>,
    filter: AppFilter,
    index: usize,
) -> Option<AppCardData> {
    sorted_apps(profile_id, apps.to_vec(), lobbies, images, covers, filter)
        .into_iter()
        .nth(index)
}

pub(crate) fn app_actions(app: &AppCardData) -> Vec<AppAction> {
    if app.status.kind == AppStatusKind::MissingImage {
        return vec![AppAction::Download];
    }

    if app.status.kind == AppStatusKind::Playing {
        return vec![AppAction::Connect, AppAction::Stop, AppAction::CheckUpdate];
    }

    vec![
        AppAction::Start,
        AppAction::StartCoop,
        AppAction::CheckUpdate,
    ]
}

pub(crate) fn filter_label(filter: AppFilter) -> &'static str {
    match filter {
        AppFilter::Default => "Default",
        AppFilter::Available => "Available",
        AppFilter::Alphabetical => "A-Z",
    }
}

fn apply_sse_event(
    profile_id: &str,
    event_type: &str,
    data: &str,
    mut lobbies: Signal<Vec<Lobby>>,
) {
    let Ok(event) = wolf_api::events::parse_event(event_type.to_string(), data.to_string()) else {
        return;
    };

    match event {
        wolf_api::events::WolfEvent::LobbyCreated(lobby)
            if lobby.started_by_profile_id == profile_id =>
        {
            lobbies.with_mut(|items| {
                items.retain(|item| item.id != lobby.id);
                items.push(*lobby);
            });
        }
        wolf_api::events::WolfEvent::LobbyStopped(lobby_id) => {
            lobbies.with_mut(|items| items.retain(|item| item.id != lobby_id));
        }
        _ => {}
    }
}

async fn image_availability(api: &wolf_api::WolfApi, apps: &[App]) -> HashMap<String, bool> {
    let images = apps
        .iter()
        .filter_map(docker_image)
        .fold(Vec::new(), |mut acc, image| {
            if !acc.contains(&image) {
                acc.push(image);
            }
            acc
        });

    let checks = images.into_iter().map(|image| async move {
        let exists = match api.docker().image_exists(&image).await {
            Ok(exists) => exists,
            Err(error) => {
                tracing::warn!(image, %error, "failed to inspect Docker image");
                true
            }
        };
        (image, exists)
    });

    futures_util::future::join_all(checks)
        .await
        .into_iter()
        .collect()
}

async fn cover_images(api: &wolf_api::WolfApi, apps: &[App]) -> HashMap<String, String> {
    let checks = apps.iter().filter_map(|app| {
        let app_id = app.id.clone();
        cover_path(app)
            .map(|path| async move { load_image_src(api, &path).await.map(|src| (app_id, src)) })
    });

    futures_util::future::join_all(checks)
        .await
        .into_iter()
        .flatten()
        .collect()
}

fn app_card_data(
    profile_id: &str,
    app: App,
    lobbies: &[Lobby],
    images: &HashMap<String, bool>,
    covers: &HashMap<String, String>,
) -> AppCardData {
    let image = docker_image(&app);
    let image_exists = image
        .as_ref()
        .and_then(|image| images.get(image))
        .copied()
        .unwrap_or(true);
    let running = lobbies
        .iter()
        .any(|lobby| is_app_lobby(profile_id, &app, lobby, false));
    let status = if running {
        AppStatus {
            kind: AppStatusKind::Playing,
            label: "Playing".to_string(),
            tone: AppStatusTone::Ready,
        }
    } else if !image_exists {
        AppStatus {
            kind: AppStatusKind::MissingImage,
            label: "Missing image".to_string(),
            tone: AppStatusTone::Warning,
        }
    } else {
        AppStatus {
            kind: AppStatusKind::Ready,
            label: "Ready".to_string(),
            tone: AppStatusTone::Ready,
        }
    };

    AppCardData {
        id: app.id.clone(),
        title: app.title.clone(),
        runner: runner_label(&app.runner).to_string(),
        source: app.clone(),
        status,
        supports_hdr: app.support_hdr,
        cover_src: covers.get(&app.id).cloned(),
    }
}

pub(crate) fn is_app_lobby(profile_id: &str, app: &App, lobby: &Lobby, multi_user: bool) -> bool {
    lobby.started_by_profile_id == profile_id
        && lobby.multi_user == multi_user
        && (lobby.name == app.title || runner_matches(&app.runner, &lobby.runner))
}

fn runner_matches(
    app_runner: &RflReflectorWolfCoreEventsAppReflTypeRunner,
    lobby_runner: &wolf_api::types::RflReflectorWolfCoreEventsLobbyReflTypeRunner,
) -> bool {
    match (app_runner, lobby_runner) {
        (
            RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppDockerTagged(app),
            wolf_api::types::RflReflectorWolfCoreEventsLobbyReflTypeRunner::WolfConfigAppDockerTagged(lobby),
        ) => app.name == lobby.name,
        (
            RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppCMDTagged(app),
            wolf_api::types::RflReflectorWolfCoreEventsLobbyReflTypeRunner::WolfConfigAppCMDTagged(lobby),
        ) => app.run_cmd == lobby.run_cmd,
        _ => false,
    }
}

pub(crate) fn docker_image(app: &App) -> Option<String> {
    docker_runner(app).map(|runner| runner.image.clone())
}

fn cover_path(app: &App) -> Option<String> {
    app.icon_png_path
        .as_ref()
        .filter(|path| !path.trim().is_empty())
        .cloned()
}

fn docker_runner(app: &App) -> Option<&WolfConfigAppDockerTagged> {
    match &app.runner {
        RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppDockerTagged(runner) => {
            Some(runner)
        }
        RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppCMDTagged(_) => None,
    }
}

fn runner_label(runner: &RflReflectorWolfCoreEventsAppReflTypeRunner) -> &'static str {
    match runner {
        RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppDockerTagged(_) => "Docker",
        RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppCMDTagged(_) => "Process",
    }
}
