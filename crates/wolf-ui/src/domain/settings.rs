use dioxus::prelude::*;
use wolf_api::apps::App;
use wolf_api::types::{RflReflectorWolfCoreEventsAppReflTypeRunner, WolfConfigAppDockerTagged};

use crate::api::ApiContext;

pub(crate) const SETTINGS_ENABLED_ENV: &str = "WOLF_UI_SETTINGS_ENABLED";
pub(crate) const AUTO_UPDATE_ENV: &str = "WOLF_UI_AUTOUPDATE";

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct WolfUiImageState {
    pub source: String,
    pub repository: String,
    pub version: String,
    pub installed: bool,
}

pub(crate) fn settings_enabled() -> bool {
    env_flag(SETTINGS_ENABLED_ENV, false)
}

pub(crate) fn auto_update_enabled() -> bool {
    env_flag(AUTO_UPDATE_ENV, false)
}

fn env_flag(key: &str, default: bool) -> bool {
    std::env::var(key)
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default)
}

pub(crate) async fn load_wolf_ui_image_state() -> Result<WolfUiImageState, String> {
    let api = ApiContext::consume();
    let response = api.apps().list().await.map_err(|error| {
        tracing::warn!(%error, "failed to load Moonlight apps for Wolf UI update settings");
        "Update settings could not load Moonlight apps from Wolf. Check that Wolf is running, then try again."
            .to_string()
    })?;

    if !response.success {
        return Err(
            "Wolf returned an unsuccessful apps response. Try again once the service is ready."
                .to_string(),
        );
    }

    let app = response
        .apps
        .iter()
        .find(|app| is_wolf_ui_app(app))
        .ok_or_else(|| detection_error(&response.apps))?;
    let image = docker_runner(app)
        .map(|runner| runner.image.clone())
        .ok_or_else(|| "The Wolf UI launcher app is not backed by a Docker image.".to_string())?;
    let installed = api.docker().image_exists(&image).await.map_err(|error| {
        tracing::warn!(image, %error, "failed to inspect configured Wolf UI image");
        "Update settings could not inspect the configured Wolf UI Docker image.".to_string()
    })?;

    Ok(WolfUiImageState {
        version: image_version(&image),
        repository: image_repository(&image),
        source: image,
        installed,
    })
}

pub(crate) async fn update_wolf_ui_image(
    image: String,
    on_progress: impl FnMut(f64),
) -> Result<bool, String> {
    let api = ApiContext::consume();
    let downloaded = api
        .docker()
        .pull_image(&image, on_progress)
        .await
        .map_err(|_| {
            "Wolf UI image update failed while pulling the configured image.".to_string()
        })?;
    let installed = api.docker().image_exists(&image).await.map_err(|_| {
        "Wolf UI image update finished, but the configured image could not be verified.".to_string()
    })?;

    if !installed {
        return Err(
            "Wolf reported pull success, but the configured image is still missing.".to_string(),
        );
    }

    Ok(downloaded)
}

pub(crate) async fn run_startup_auto_update() -> Result<bool, String> {
    let state = load_wolf_ui_image_state().await?;
    update_wolf_ui_image(state.source, |_| {}).await
}

fn is_wolf_ui_app(app: &App) -> bool {
    let Some(runner) = docker_runner(app) else {
        return false;
    };

    let haystack =
        format!("{} {} {} {}", app.title, app.id, runner.name, runner.image).to_ascii_lowercase();

    haystack.contains("wolf-ui")
        || haystack.contains("wolf ui")
        || runner.image.to_ascii_lowercase().ends_with("/wolf-ui")
        || runner.image.to_ascii_lowercase().contains("/wolf-ui:")
}

fn detection_error(apps: &[App]) -> String {
    if apps.is_empty() {
        return "Update settings could not find the Wolf UI Docker app because Wolf returned no Moonlight apps."
            .to_string();
    }

    format!(
        "Update settings could not find the Wolf UI Docker app. Moonlight apps returned: {}.",
        apps.iter().map(app_summary).collect::<Vec<_>>().join(", ")
    )
}

fn app_summary(app: &App) -> String {
    match docker_runner(app) {
        Some(runner) => format!("{} ({})", app.title, runner.image),
        None => format!("{} (process)", app.title),
    }
}

fn docker_runner(app: &App) -> Option<&WolfConfigAppDockerTagged> {
    match &app.runner {
        RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppDockerTagged(runner) => {
            Some(runner)
        }
        RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppCMDTagged(_) => None,
    }
}

fn image_version(image: &str) -> String {
    let tag_index = image.rfind(':');
    let slash_index = image.rfind('/');

    match tag_index {
        Some(tag_index) if slash_index.is_none_or(|slash_index| tag_index > slash_index) => {
            image[tag_index + 1..].to_string()
        }
        _ => "latest".to_string(),
    }
}

fn image_repository(image: &str) -> String {
    let tag_index = image.rfind(':');
    let slash_index = image.rfind('/');

    match tag_index {
        Some(tag_index) if slash_index.is_none_or(|slash_index| tag_index > slash_index) => {
            image[..tag_index].to_string()
        }
        _ => image.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use wolf_api::types::{RflReflectorWolfCoreEventsAppReflType, WolfConfigAppCMDTagged};

    use super::*;

    #[test]
    fn env_flag_uses_default_when_unset() {
        assert!(!env_flag("WOLF_UI_TEST_FLAG_MISSING", false));
        assert!(env_flag("WOLF_UI_TEST_FLAG_MISSING", true));
    }

    #[test]
    fn env_flag_parses_truthy_values() {
        for value in ["1", "true", "TRUE", "Yes", "on"] {
            unsafe { std::env::set_var("WOLF_UI_TEST_FLAG", value) };
            assert!(
                env_flag("WOLF_UI_TEST_FLAG", false),
                "{value} should be true"
            );
        }

        unsafe { std::env::remove_var("WOLF_UI_TEST_FLAG") };
    }

    #[test]
    fn env_flag_parses_other_values_as_false() {
        for value in ["0", "false", "off", ""] {
            unsafe { std::env::set_var("WOLF_UI_TEST_FLAG", value) };
            assert!(
                !env_flag("WOLF_UI_TEST_FLAG", true),
                "{value} should be false"
            );
        }

        unsafe { std::env::remove_var("WOLF_UI_TEST_FLAG") };
    }

    #[test]
    fn detects_wolf_ui_from_title_runner_name_or_image() {
        assert!(is_wolf_ui_app(&docker_app(
            "launcher",
            "Wolf UI",
            "custom-name",
            "registry.local/custom:main"
        )));
        assert!(is_wolf_ui_app(&docker_app(
            "launcher",
            "Launcher",
            "Wolf-UI",
            "registry.local/custom:main"
        )));
        assert!(is_wolf_ui_app(&docker_app(
            "launcher",
            "Launcher",
            "custom-name",
            "ghcr.io/games-on-whales/wolf-ui:main"
        )));
        assert!(!is_wolf_ui_app(&docker_app(
            "steam",
            "Steam",
            "WolfSteam",
            "ghcr.io/games-on-whales/steam:edge"
        )));
        assert!(!is_wolf_ui_app(&cmd_app(
            "wolf-ui-helper",
            "Wolf UI Helper"
        )));
    }

    #[test]
    fn extracts_tag_without_treating_registry_port_as_version() {
        assert_eq!(
            image_version("ghcr.io/games-on-whales/wolf-ui:main"),
            "main"
        );
        assert_eq!(image_version("localhost:5000/wolf-ui"), "latest");
        assert_eq!(image_version("localhost:5000/wolf-ui:dev"), "dev");
    }

    #[test]
    fn extracts_repository_without_treating_registry_port_as_tag() {
        assert_eq!(
            image_repository("ghcr.io/games-on-whales/wolf-ui:main"),
            "ghcr.io/games-on-whales/wolf-ui"
        );
        assert_eq!(
            image_repository("localhost:5000/wolf-ui"),
            "localhost:5000/wolf-ui"
        );
        assert_eq!(
            image_repository("localhost:5000/wolf-ui:dev"),
            "localhost:5000/wolf-ui"
        );
    }

    fn docker_app(
        id: &str,
        title: &str,
        name: &str,
        image: &str,
    ) -> RflReflectorWolfCoreEventsAppReflType {
        RflReflectorWolfCoreEventsAppReflType {
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

    fn cmd_app(id: &str, title: &str) -> RflReflectorWolfCoreEventsAppReflType {
        RflReflectorWolfCoreEventsAppReflType {
            av1_gst_pipeline: String::new(),
            h264_gst_pipeline: String::new(),
            hevc_gst_pipeline: String::new(),
            icon_png_path: None,
            id: id.to_string(),
            opus_gst_pipeline: String::new(),
            render_node: String::new(),
            runner: RflReflectorWolfCoreEventsAppReflTypeRunner::WolfConfigAppCMDTagged(
                WolfConfigAppCMDTagged {
                    run_cmd: String::new(),
                },
            ),
            start_audio_server: false,
            start_virtual_compositor: false,
            support_hdr: false,
            title: title.to_string(),
        }
    }
}
