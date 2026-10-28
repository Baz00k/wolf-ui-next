use dioxus::prelude::*;
use tw_merge::tw_merge;

use crate::components::ImagePullProgress;
use crate::components::primitives::{
    Button, ButtonSize, Card, CardContent, Spinner, StatusAlert, StatusAlertVariant,
};
use crate::domain::image_pulls::{PullKind, PullOutcome, use_image_pulls};
use crate::domain::settings::{WOLF_UI_IMAGE_TITLE, WolfUiImageState, load_wolf_ui_image_state};

#[component]
pub fn SettingsImageUpdates() -> Element {
    let mut image_state = use_resource(load_wolf_ui_image_state);

    rsx! {
        match &*image_state.read_unchecked() {
            Some(Ok(state)) => rsx! {
                UpdatePanel { state: state.clone() }
            },
            Some(Err(message)) => rsx! {
                StatusAlert {
                    title: Some("Update settings unavailable".to_string()),
                    message: message.clone(),
                    variant: StatusAlertVariant::Error,
                    Button { size: ButtonSize::Lg, onclick: move |_| image_state.restart(), "Retry" }
                }
            },
            None => rsx! {
                div { class: "w-full h-full grid place-items-center",
                    Spinner { class: "m-8 h-8 w-8" }
                }
            },
        }
    }
}

#[component]
fn UpdatePanel(state: WolfUiImageState) -> Element {
    let pulls = use_image_pulls();
    let pull = pulls.kind(&state.source);
    let outcome = pulls.outcome(&state.source);

    rsx! {
        Card { class: "overflow-hidden rounded-xl bg-card",
            CardContent { class: "space-y-6 px-6 py-6 sm:px-10 sm:py-8",
                PageHeader {}
                div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4",
                    ImageInfoTile {
                        label: "Current image source".to_string(),
                        value: state.repository.clone(),
                        wide: true,
                    }
                    ImageInfoTile {
                        label: "Current image version".to_string(),
                        value: state.version.clone(),
                    }
                    ImageInfoTile {
                        label: "Image status".to_string(),
                        value: image_status(state.installed, outcome).to_string(),
                        failed: outcome == Some(PullOutcome::Failed),
                    }
                }
                UpdateAction { image: state.source.clone(), pull }
            }
        }
    }
}

#[component]
fn PageHeader() -> Element {
    rsx! {
        h2 { class: "text-3xl font-bold tracking-tight sm:text-4xl", "Update" }
    }
}

#[component]
fn ImageInfoTile(
    label: String,
    value: String,
    #[props(default)] wide: bool,
    #[props(default)] failed: bool,
) -> Element {
    let class = tw_merge!(
        "rounded-lg border border-border/80 bg-background/35 p-5",
        if wide {
            "sm:col-span-2 lg:col-span-2"
        } else {
            ""
        },
    );
    let value_class = tw_merge!(
        "mt-3 break-words text-2xl font-bold tracking-tight text-foreground",
        if failed {
            "text-destructive-foreground"
        } else {
            ""
        },
    );

    rsx! {
        div { class,
            p { class: "text-sm font-semibold uppercase tracking-widest text-muted-foreground",
                "{label}"
            }
            p { class: value_class, "{value}" }
        }
    }
}

#[component]
fn UpdateAction(image: String, pull: Option<PullKind>) -> Element {
    let pulls = use_image_pulls();

    rsx! {
        div { class: "flex flex-col gap-4",
            div { class: "flex justify-center sm:justify-end",
                Button {
                    size: ButtonSize::Xl,
                    class: "font-bold",
                    action_label: "Update image",
                    disabled: pull.is_some(),
                    onclick: {
                        let image = image.clone();
                        move |_| {
                            pulls.start(image.clone(), PullKind::Update, WOLF_UI_IMAGE_TITLE.to_string());
                        }
                    },
                    if pull.is_some() {
                        Spinner { class: "h-5 w-5" }
                        "Updating"
                    } else {
                        "Update Image"
                    }
                }
            }
            if let Some(kind) = pull {
                ImagePullProgress { image, kind }
            }
        }
    }
}

fn image_status(installed: bool, outcome: Option<PullOutcome>) -> &'static str {
    match outcome {
        Some(PullOutcome::Downloaded) => "Restart to apply",
        Some(PullOutcome::UpToDate) => "Up to date",
        Some(PullOutcome::Failed) => "Update failed",
        None if installed => "Installed",
        None => "Missing",
    }
}
