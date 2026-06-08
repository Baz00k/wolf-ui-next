use dioxus::prelude::*;

mod api;
mod components;
mod domain;
mod input;
mod views;

use api::ApiContext;
use views::{
    AppLayout, AppSettings, ProfileApps, ProfileSettings, Profiles, Settings, ThemeSettings,
};

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    Profiles {},

    #[route("/profiles/:profile_id/apps")]
    ProfileApps { profile_id: String },

    #[route("/settings")]
    Settings {},

    #[route("/settings/profile")]
    ProfileSettings {},

    #[route("/settings/app")]
    AppSettings {},

    #[route("/settings/theme")]
    ThemeSettings {},
}

const STYLES_CSS: Asset = asset!("/assets/dist/styles.css");
const SCROLL_ANIMATION_JS: Asset = asset!("/assets/scroll-animation.js");
const FOCUS_NAVIGATION_JS: Asset = asset!("/assets/focus-navigation.js");

fn main() {
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

#[component]
fn App() -> Element {
    ApiContext::provide();
    components::use_toast_provider();

    rsx! {
        document::Stylesheet { href: STYLES_CSS }
        script { src: SCROLL_ANIMATION_JS }
        script { src: FOCUS_NAVIGATION_JS }
        input::InputProvider {
            Router::<Route> {}
        }
    }
}
