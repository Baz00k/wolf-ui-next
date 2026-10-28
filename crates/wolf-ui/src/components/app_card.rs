use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::hi_solid_icons::HiCube;

use crate::components::primitives::{
    Badge, BadgeVariant, Card, CardTrigger, Progress, ProgressTone, Skeleton,
};
use crate::domain::apps::docker_image;
use crate::domain::image_pulls::{PullKind, use_image_pulls};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppStatusKind {
    Ready,
    Playing,
    MissingImage,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppStatusTone {
    Ready,
    Warning,
}

#[derive(Clone, PartialEq, Eq)]
pub struct AppStatus {
    pub kind: AppStatusKind,
    pub label: String,
    pub tone: AppStatusTone,
}

#[derive(Clone, PartialEq)]
pub struct AppCardData {
    pub id: String,
    pub title: String,
    pub runner: String,
    pub source: wolf_api::profiles::App,
    pub status: AppStatus,
    pub supports_hdr: bool,
    pub cover_src: Option<String>,
}

#[component]
pub fn AppCard(
    app: AppCardData,
    index: usize,
    autofocus: bool,
    #[props(default)] pull: Option<PullKind>,
    onfocus: EventHandler<FocusEvent>,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let pull =
        pull.and_then(|kind| docker_image(&app.source).map(|image| (kind, image.to_string())));
    let show_runner = app.runner != "Docker";
    let is_unavailable = app.status.kind == AppStatusKind::MissingImage;
    let is_playing = app.status.kind == AppStatusKind::Playing;

    rsx! {
        CardTrigger {
            class: "aspect-auto flex-col text-left",
            action_label: "Actions",
            index,
            autofocus,
            onfocus,
            onclick,
            Card { class: "relative aspect-[3/4] w-full overflow-visible border-border/70 text-muted-foreground opacity-80 duration-300 ease-out group-focus-visible:text-card-foreground group-focus-visible:opacity-100",
                div { class: "pointer-events-none absolute inset-x-4 top-4 z-20 flex items-start justify-between gap-3 text-sm font-semibold uppercase tracking-wider text-muted-foreground sm:inset-x-6 sm:top-6",
                    if show_runner {
                        span { "{app.runner}" }
                    } else {
                        span {}
                    }
                    div { class: "flex flex-col items-end gap-2",
                        if app.supports_hdr {
                            Badge { variant: BadgeVariant::Warning, class: "px-2", "HDR" }
                        }
                        if let Some((kind, image)) = pull.clone() {
                            PullBadge { image, kind }
                        } else if is_unavailable {
                            StatusBadge { label: "Not installed" }
                        }
                        if is_playing {
                            StatusBadge { label: "Playing", pulse: true }
                        }
                    }
                }
                div { class: "relative flex h-full items-center justify-center overflow-hidden rounded-lg",
                    if let Some(cover_src) = app.cover_src.as_ref() {
                        img {
                            class: "absolute inset-0 h-full w-full rounded-lg object-cover transition duration-300",
                            class: if is_unavailable { "grayscale brightness-40" },
                            src: cover_src.clone(),
                            loading: "lazy",
                            draggable: "false",
                        }
                    } else {
                        // Without art the card is unrecognisable, so keep the
                        // name on it for inputs that never focus or hover.
                        div { class: "flex flex-col items-center gap-4 px-6 text-center",
                            div { class: "flex h-20 w-20 items-center justify-center text-muted-foreground sm:h-32 sm:w-32 md:h-36 md:w-36",
                                Icon {
                                    icon: HiCube,
                                    class: "w-24 h-24",
                                    width: None,
                                    height: None,
                                }
                            }
                            p { class: "line-clamp-2 text-2xl font-bold leading-tight tracking-tight",
                                "{app.title}"
                            }
                        }
                    }
                    if let Some((_, image)) = pull {
                        PullProgressBar { image }
                    }
                }
            }
            h2 { class: "mt-4 h-8 truncate px-2 text-2xl font-bold leading-8 tracking-tight sm:h-9 sm:text-3xl sm:leading-9 text-foreground opacity-0 group-focus-visible:opacity-100",
                "{app.title}"
            }
        }
    }
}

#[component]
fn StatusBadge(label: &'static str, #[props(default)] pulse: bool) -> Element {
    rsx! {
        Badge { variant: BadgeVariant::Overlay, class: "gap-2 py-1.5",
            if pulse {
                span { class: "h-2 w-2 animate-pulse rounded-full bg-emerald-400" }
            }
            span { class: "text-sm font-semibold uppercase tracking-wider text-muted-foreground",
                "{label}"
            }
        }
    }
}

#[component]
fn PullBadge(image: String, kind: PullKind) -> Element {
    let progress = use_image_pulls().progress(&image);

    rsx! {
        Badge { variant: BadgeVariant::Warning, class: "gap-2 bg-black/55 py-1.5 backdrop-blur-sm",
            span { class: "h-2 w-2 animate-pulse rounded-full bg-yellow-300" }
            span { class: "tabular-nums", "{kind.progress_label()} {progress}%" }
        }
    }
}

#[component]
fn PullProgressBar(image: String) -> Element {
    let progress = use_image_pulls().progress(&image);

    rsx! {
        Progress {
            value: progress,
            tone: ProgressTone::Warning,
            class: "pointer-events-none absolute inset-x-4 bottom-4 z-10 h-2 bg-black/60 sm:inset-x-6 sm:bottom-6",
        }
    }
}

#[component]
pub fn AppCardSkeleton() -> Element {
    rsx! {
        div { class: "flex w-full flex-col",
            article { class: "flex aspect-[3/4] w-full animate-pulse flex-col overflow-hidden rounded-xl border border-border bg-card/60 opacity-70",
                div { class: "relative flex flex-1 items-center justify-center rounded-lg",
                    Skeleton { class: "absolute inset-0 rounded-lg" }
                    div { class: "absolute inset-x-4 top-4 z-20 flex items-start justify-between gap-3 sm:inset-x-6 sm:top-6",
                        Skeleton { class: "h-5 w-16 rounded-full" }
                        Skeleton { class: "h-7 w-28 rounded-full" }
                    }
                    Skeleton { class: "h-20 w-20 rounded-xl sm:h-32 sm:w-32 md:h-36 md:w-36" }
                }
            }
            // Reserve the caption row so loaded cards don't shift the grid.
            div { class: "mt-4 h-8 sm:h-9" }
        }
    }
}

#[cfg(test)]
mod tests {
    use wolf_api::types::{AppDocker, AppDockerType, Runner};

    use super::*;
    use crate::components::primitives::use_toast_provider;
    use crate::domain::image_pulls::use_image_pulls_provider;

    #[component]
    fn TestCard(
        status: AppStatusKind,
        pull: Option<PullKind>,
        #[props(default)] cover_src: Option<String>,
    ) -> Element {
        use_toast_provider();
        use_image_pulls_provider();

        rsx! {
            AppCard {
                app: AppCardData {
                    cover_src,
                    ..steam(status)
                },
                index: 0,
                autofocus: false,
                pull,
                onfocus: |_| {},
                onclick: |_| {},
            }
        }
    }

    fn render(status: AppStatusKind, pull: Option<PullKind>) -> String {
        render_card(TestCardProps {
            status,
            pull,
            cover_src: None,
        })
    }

    fn render_card(props: TestCardProps) -> String {
        let mut dom = VirtualDom::new_with_props(TestCard, props);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn cover_card_shows_title_only_in_focus_caption() {
        let html = render_card(TestCardProps {
            status: AppStatusKind::Ready,
            pull: None,
            cover_src: Some("cover.png".to_string()),
        });

        assert_eq!(html.matches(">Steam<").count(), 1);
        assert!(html.contains("group-focus-visible:opacity-100"));
    }

    #[test]
    fn coverless_card_keeps_title_on_card() {
        let html = render(AppStatusKind::Ready, None);

        assert_eq!(html.matches(">Steam<").count(), 2);
    }

    #[test]
    fn pulling_card_shows_progress_badge() {
        let html = render(AppStatusKind::MissingImage, Some(PullKind::Download));

        assert!(html.contains("Downloading 0%"));
        assert!(html.contains(r#"data-slot="progress""#));
        assert!(!html.contains("Not installed"));
    }

    #[test]
    fn playing_card_keeps_playing_badge_while_updating() {
        let html = render(AppStatusKind::Playing, Some(PullKind::Update));

        assert!(html.contains("Updating 0%"));
        assert!(html.contains("Playing"));
    }

    #[test]
    fn idle_card_has_no_pull_progress() {
        let html = render(AppStatusKind::MissingImage, None);

        assert!(html.contains("Not installed"));
        assert!(!html.contains(r#"data-slot="progress""#));
    }

    fn steam(kind: AppStatusKind) -> AppCardData {
        AppCardData {
            id: "steam".to_string(),
            title: "Steam".to_string(),
            runner: "Docker".to_string(),
            source: wolf_api::profiles::App {
                av1_gst_pipeline: String::new(),
                h264_gst_pipeline: String::new(),
                hevc_gst_pipeline: String::new(),
                icon_png_path: None,
                id: "steam".to_string(),
                opus_gst_pipeline: String::new(),
                render_node: String::new(),
                runner: Runner::Docker(AppDocker {
                    base_create_json: None,
                    devices: Vec::new(),
                    env: Vec::new(),
                    image: "ghcr.io/wolf/steam".to_string(),
                    mounts: Vec::new(),
                    name: "steam".to_string(),
                    ports: Vec::new(),
                    type_: AppDockerType::Docker,
                }),
                start_audio_server: false,
                start_virtual_compositor: false,
                support_hdr: false,
                title: "Steam".to_string(),
            },
            status: AppStatus {
                kind,
                label: String::new(),
                tone: AppStatusTone::Ready,
            },
            supports_hdr: false,
            cover_src: None,
        }
    }
}
