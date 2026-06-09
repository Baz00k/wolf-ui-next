use std::collections::HashMap;

use dioxus::prelude::*;
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
    let response = api.profiles().apps(&profile_id).await.map_err(|_| {
        "Apps could not be loaded. Check that Wolf is running, then try again.".to_string()
    })?;
    let lobby_response = api.lobbies().list().await.map_err(|_| {
        "Apps could not be loaded. Check that Wolf is running, then try again.".to_string()
    })?;
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
    let _ = ApiContext::consume()
        .events()
        .listen(|event| apply_wolf_event(&profile_id, event, lobbies))
        .await;
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

fn apply_wolf_event(
    profile_id: &str,
    event: wolf_api::events::WolfEvent,
    mut lobbies: Signal<Vec<Lobby>>,
) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use wolf_api::types::{RflReflectorWolfCoreEventsLobbyReflTypeRunner, WolfConfigAppCMDTagged};

    #[test]
    fn sorted_apps_marks_running_and_missing_image_states() {
        let apps = vec![
            docker_app("steam", "Steam", "steam", "ghcr.io/wolf/steam"),
            cmd_app("retroarch", "RetroArch", "retroarch"),
        ];
        let lobbies = vec![lobby(
            "lobby-1",
            "profile-1",
            "Steam",
            docker_lobby_runner("steam"),
            false,
        )];
        let images = HashMap::from([("ghcr.io/wolf/steam".to_string(), false)]);
        let apps = sorted_apps(
            "profile-1",
            apps,
            &lobbies,
            &images,
            &HashMap::new(),
            AppFilter::Default,
        );

        assert!(matches!(apps[0].status.kind, AppStatusKind::Playing));
        assert!(matches!(apps[1].status.kind, AppStatusKind::Ready));
    }

    #[test]
    fn unavailable_apps_only_offer_download_action() {
        let apps = vec![docker_app("steam", "Steam", "steam", "ghcr.io/wolf/steam")];
        let images = HashMap::from([("ghcr.io/wolf/steam".to_string(), false)]);
        let app = selected_app_data(
            "profile-1",
            &apps,
            &[],
            &images,
            &HashMap::new(),
            AppFilter::Default,
            0,
        )
        .expect("app is present");

        assert!(matches!(app.status.kind, AppStatusKind::MissingImage));
        assert_eq!(app_actions(&app), vec![AppAction::Download]);
    }

    #[test]
    fn lobbies_match_by_profile_mode_and_runner_identity() {
        let app = docker_app("steam", "Steam Big Picture", "steam", "ghcr.io/wolf/steam");
        let lobby = lobby(
            "lobby-1",
            "profile-1",
            "Different title",
            docker_lobby_runner("steam"),
            false,
        );

        assert!(is_app_lobby("profile-1", &app, &lobby, false));
        assert!(!is_app_lobby("other-profile", &app, &lobby, false));
        assert!(!is_app_lobby("profile-1", &app, &lobby, true));
    }

    fn docker_app(id: &str, title: &str, name: &str, image: &str) -> App {
        App {
            av1_gst_pipeline: String::new(),
            h264_gst_pipeline: String::new(),
            hevc_gst_pipeline: String::new(),
            icon_png_path: None,
            id: id.to_string(),
            opus_gst_pipeline: String::new(),
            render_node: String::new(),
            runner: RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppDockerTagged(
                WolfConfigAppDockerTagged {
                    base_create_json: None,
                    devices: Vec::new(),
                    env: Vec::new(),
                    image: image.to_string(),
                    mounts: Vec::new(),
                    name: name.to_string(),
                    ports: Vec::new(),
                },
            ),
            start_audio_server: false,
            start_virtual_compositor: false,
            support_hdr: false,
            title: title.to_string(),
        }
    }

    fn cmd_app(id: &str, title: &str, run_cmd: &str) -> App {
        App {
            av1_gst_pipeline: String::new(),
            h264_gst_pipeline: String::new(),
            hevc_gst_pipeline: String::new(),
            icon_png_path: None,
            id: id.to_string(),
            opus_gst_pipeline: String::new(),
            render_node: String::new(),
            runner: RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppCMDTagged(
                WolfConfigAppCMDTagged {
                    run_cmd: run_cmd.to_string(),
                },
            ),
            start_audio_server: false,
            start_virtual_compositor: false,
            support_hdr: false,
            title: title.to_string(),
        }
    }

    fn lobby(
        id: &str,
        profile_id: &str,
        name: &str,
        runner: RflReflectorWolfCoreEventsLobbyReflTypeRunner,
        multi_user: bool,
    ) -> Lobby {
        Lobby {
            connected_sessions: Vec::new(),
            icon_png_path: None,
            id: id.to_string(),
            multi_user,
            name: name.to_string(),
            pin_required: false,
            runner,
            started_by_profile_id: profile_id.to_string(),
            stop_when_everyone_leaves: false,
        }
    }

    fn docker_lobby_runner(name: &str) -> RflReflectorWolfCoreEventsLobbyReflTypeRunner {
        RflReflectorWolfCoreEventsLobbyReflTypeRunner::WolfConfigAppDockerTagged(
            WolfConfigAppDockerTagged {
                base_create_json: None,
                devices: Vec::new(),
                env: Vec::new(),
                image: String::new(),
                mounts: Vec::new(),
                name: name.to_string(),
                ports: Vec::new(),
            },
        )
    }
}
