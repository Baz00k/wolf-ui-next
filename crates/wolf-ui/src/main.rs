use dioxus::prelude::*;

mod api;
mod components;
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

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    ApiContext::provide();

    rsx! {
        document::Stylesheet { href: STYLES_CSS }
        Router::<Route> {}
    }
}
