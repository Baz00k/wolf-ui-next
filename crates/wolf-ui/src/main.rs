use dioxus::prelude::*;

mod api;
mod components;
mod domain;
mod input;
mod views;

use api::ApiContext;
use components::StartupAutoUpdate;
use components::primitives::ToastViewport;
use views::{AppLayout, ProfileApps, Profiles, Settings, SettingsImageUpdates, SettingsLayout};

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    Profiles {},

    #[route("/profiles/:profile_id/apps")]
    ProfileApps { profile_id: String },

    #[layout(SettingsLayout)]
    #[route("/settings")]
    Settings {},

    #[route("/settings/image-updates")]
    SettingsImageUpdates {},
}

const STYLES_CSS: Asset = asset!("/assets/dist/styles.css");
const SCROLL_ANIMATION_JS: Asset = asset!("/assets/scroll-animation.js");
const FOCUS_NAVIGATION_JS: Asset = asset!("/assets/focus-navigation.js");
const UI_SOUNDS_JS: Asset = asset!("/assets/ui-sounds.js");
const NAVIGATION_SOUND: Asset = asset!("/assets/audio/navigation.ogg");
const SELECT_SOUND: Asset = asset!("/assets/audio/select.ogg");
const BACK_SOUND: Asset = asset!("/assets/audio/back.ogg");

fn main() {
    init_logging();

    let mut window = dioxus::desktop::WindowBuilder::new().with_title("Wolf UI");

    if !cfg!(debug_assertions) {
        window = window.with_decorations(false);
    }

    let mut config = dioxus::desktop::Config::new()
        .with_window(window)
        .with_background_color((0, 0, 0, 255));

    if !cfg!(debug_assertions) {
        config = config.with_on_window(|window, _| {
            window.set_fullscreen(Some(dioxus::desktop::tao::window::Fullscreen::Borderless(
                None,
            )));
        });
    }

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}

fn init_logging() {
    use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("wolf_ui=info,wolf_api=info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_target(true))
        .init();
}

#[component]
fn App() -> Element {
    ApiContext::provide();
    components::primitives::use_toast_provider();

    rsx! {
        document::Stylesheet { href: STYLES_CSS }
        script {
            "window.__wolfUiSoundUrls = {{ navigate: '{NAVIGATION_SOUND}', select: '{SELECT_SOUND}', back: '{BACK_SOUND}' }};"
        }
        script { src: UI_SOUNDS_JS }
        script { src: SCROLL_ANIMATION_JS }
        script { src: FOCUS_NAVIGATION_JS }
        input::InputProvider { Router::<Route> {} }
        ToastViewport {}
        StartupAutoUpdate {}
    }
}
