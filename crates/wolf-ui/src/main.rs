use dioxus::prelude::*;
use wolf_api::{self, profiles::Profile};

#[derive(Clone)]
struct ApiContext {
    api: wolf_api::WolfApi,
}

impl ApiContext {
    fn provide() {
        use_context_provider(|| Self {
            api: wolf_api::client().expect("build Wolf API client"),
        });
    }

    fn consume() -> wolf_api::WolfApi {
        consume_context::<Self>().api
    }
}

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
    ApiContext::provide();

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: STYLES_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Profiles() -> Element {
    let profiles = use_resource(move || async move {
        ApiContext::consume()
            .profiles()
            .list()
            .await
            .map_err(|error| error.to_string())
    });

    rsx! {
        main { class: "min-h-screen bg-zinc-950 text-zinc-100 p-8",
            div { class: "mx-auto max-w-5xl space-y-6",
                header { class: "space-y-2",
                    p { class: "text-sm font-medium uppercase tracking-[0.25em] text-cyan-400", "Wolf" }
                    h1 { class: "text-4xl font-semibold tracking-tight", "Profiles" }
                    p { class: "text-zinc-400", "Choose a profile to inspect its available apps." }
                }

                match &*profiles.read_unchecked() {
                    Some(Ok(response)) => rsx! { ProfileList { profiles: response.profiles.clone() } },
                    Some(Err(error)) => rsx! { ErrorPanel { message: error.clone() } },
                    None => rsx! { LoadingPanel { label: "Loading profiles" } },
                }
            }
        }
    }
}

#[component]
fn ProfileList(profiles: Vec<Profile>) -> Element {
    if profiles.is_empty() {
        return rsx! {
            EmptyPanel { title: "No profiles found", body: "Wolf did not return any profiles." }
        };
    }

    rsx! {
        section { class: "grid gap-4 md:grid-cols-2",
            for profile in profiles {
                Link {
                    class: "group rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5 transition hover:border-cyan-500/60 hover:bg-zinc-900",
                    to: Route::Apps { profile_id: profile.id.clone() },
                    div { class: "flex items-start justify-between gap-4",
                        div { class: "space-y-2",
                            h2 { class: "text-xl font-semibold", "{profile.name}" }
                            p { class: "text-sm text-zinc-500", "{profile.id}" }
                        }
                        span { class: "rounded-full bg-zinc-800 px-3 py-1 text-sm text-zinc-300 group-hover:bg-cyan-500/10 group-hover:text-cyan-300",
                            "{profile.apps.len()} apps"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Apps(profile_id: String) -> Element {
    let profile_id_for_query = profile_id.clone();
    let apps = use_resource(move || {
        let profile_id = profile_id_for_query.clone();
        async move {
            ApiContext::consume()
                .profiles()
                .apps(&profile_id)
                .await
                .map_err(|error| error.to_string())
        }
    });

    rsx! {
        main { class: "min-h-screen bg-zinc-950 text-zinc-100 p-8",
            div { class: "mx-auto max-w-5xl space-y-6",
                header { class: "space-y-3",
                    Link { class: "text-sm text-cyan-400 hover:text-cyan-300", to: Route::Profiles {}, "← Profiles" }
                    h1 { class: "text-4xl font-semibold tracking-tight", "Profile apps" }
                    p { class: "text-zinc-400", "Profile ID: {profile_id}" }
                }

                match &*apps.read_unchecked() {
                    Some(Ok(response)) => rsx! { AppList { apps: response.apps.clone() } },
                    Some(Err(error)) => rsx! { ErrorPanel { message: error.clone() } },
                    None => rsx! { LoadingPanel { label: "Loading apps" } },
                }
            }
        }
    }
}

#[component]
fn AppList(apps: Vec<wolf_api::profiles::App>) -> Element {
    if apps.is_empty() {
        return rsx! {
            EmptyPanel { title: "No apps found", body: "This profile has no apps attached." }
        };
    }

    rsx! {
        section { class: "grid gap-4 md:grid-cols-2",
            for app in apps {
                article { class: "rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5",
                    div { class: "space-y-2",
                        h2 { class: "text-xl font-semibold", "{app.title}" }
                        p { class: "text-sm text-zinc-500", "{app.id}" }
                    }
                    div { class: "mt-4 flex flex-wrap gap-2 text-xs text-zinc-400",
                        if app.support_hdr {
                            span { class: "rounded-full bg-cyan-500/10 px-2 py-1 text-cyan-300", "HDR" }
                        }
                        if app.start_virtual_compositor {
                            span { class: "rounded-full bg-zinc-800 px-2 py-1", "Virtual compositor" }
                        }
                        if app.start_audio_server {
                            span { class: "rounded-full bg-zinc-800 px-2 py-1", "Audio server" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Settings() -> Element {
    rsx! {
        main { class: "min-h-screen bg-zinc-950 text-zinc-100 p-8",
            div { class: "mx-auto max-w-5xl",
                h1 { class: "text-4xl font-semibold tracking-tight", "Settings" }
            }
        }
    }
}

#[component]
fn LoadingPanel(label: String) -> Element {
    rsx! {
        div { class: "rounded-2xl border border-zinc-800 bg-zinc-900/70 p-6 text-zinc-300",
            "{label}…"
        }
    }
}

#[component]
fn ErrorPanel(message: String) -> Element {
    rsx! {
        div { class: "rounded-2xl border border-red-900/70 bg-red-950/40 p-6 text-red-200",
            h2 { class: "font-semibold", "Could not fetch Wolf data" }
            p { class: "mt-2 text-sm text-red-200/80", "{message}" }
        }
    }
}

#[component]
fn EmptyPanel(title: String, body: String) -> Element {
    rsx! {
        div { class: "rounded-2xl border border-zinc-800 bg-zinc-900/70 p-6 text-zinc-300",
            h2 { class: "font-semibold text-zinc-100", "{title}" }
            p { class: "mt-2 text-sm text-zinc-400", "{body}" }
        }
    }
}
