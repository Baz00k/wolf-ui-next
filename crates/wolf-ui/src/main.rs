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
        window = window.with_decorations(false).with_fullscreen(Some(
            dioxus::desktop::tao::window::Fullscreen::Borderless(None),
        ));
    }

    let mut config = dioxus::desktop::Config::new().with_window(window);

    if !cfg!(debug_assertions) {
        config = config.with_on_window(|window, _| {
            let monitor = window
                .current_monitor()
                .or_else(|| window.available_monitors().next());

            if let Some(monitor) = monitor {
                window.set_inner_size(monitor.size());
            }
        });
    }

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}

#[component]
fn App() -> Element {
    ApiContext::provide();

    use_effect(|| {
        log_startup_screen_info();
    });

    rsx! {
        document::Stylesheet { href: STYLES_CSS }
        script { src: SCROLL_ANIMATION_JS }
        script { src: FOCUS_NAVIGATION_JS }
        input::InputProvider {
            Router::<Route> {}
        }
    }
}

fn log_startup_screen_info() {
    let window = dioxus::desktop::window();
    let inner_size = window.inner_size();
    let outer_size = window.outer_size();
    let scale_factor = window.scale_factor();
    let fullscreen = window.fullscreen();

    tracing::debug!(
        target: "wolf-ui-screen",
        "window inner={}x{} outer={}x{} scale_factor={} fullscreen={:?}",
        inner_size.width,
        inner_size.height,
        outer_size.width,
        outer_size.height,
        scale_factor,
        fullscreen,
    );

    match window.current_monitor() {
        Some(monitor) => log_monitor("current_monitor", &monitor),
        None => tracing::debug!(
            target: "wolf-ui-screen",
            "current_monitor=None"
        ),
    }

    match window.primary_monitor() {
        Some(monitor) => log_monitor("primary_monitor", &monitor),
        None => tracing::debug!(
            target: "wolf-ui-screen",
            "primary_monitor=None"
        ),
    }

    for (index, monitor) in window.available_monitors().enumerate() {
        log_monitor(&format!("available_monitor[{index}]"), &monitor);
    }
}

fn log_monitor(label: &str, monitor: &dioxus::desktop::tao::monitor::MonitorHandle) {
    let size = monitor.size();
    let position = monitor.position();
    let scale_factor = monitor.scale_factor();

    tracing::debug!(
        target: "wolf-ui-screen",
        "{} name={:?} size={}x{} position={},{} scale_factor={}",
        label,
        monitor.name(),
        size.width,
        size.height,
        position.x,
        position.y,
        scale_factor,
    );
}
