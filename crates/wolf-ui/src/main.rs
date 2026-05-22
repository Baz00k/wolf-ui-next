use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[route("/")]
    Profiles {},

    #[route("/profiles/:profile_id/apps")]
    Apps { profile_id: String },

    #[route("/settings")]
    Settings {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const STYLES_CSS: Asset = asset!("/assets/dist/styles.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: STYLES_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Profiles() -> Element {
    rsx! {
        h1 { "Profiles" }
    }
}

#[component]
fn Apps(profile_id: String) -> Element {
    rsx! {
        h1 { "Apps" }
        p { "Profile ID: {profile_id}" }
    }
}

#[component]
fn Settings() -> Element {
    rsx! {
        h1 { "Settings" }
    }
}
