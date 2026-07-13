use dioxus::prelude::*;
use tw_merge::tw_merge;

use crate::components::primitives::{
    Button, ButtonSize, Card, CardContent, ProgressPanel, Spinner, StatusAlert, StatusAlertVariant,
    ToastContext, ToastOptions, use_toasts,
};
use crate::domain::settings::{WolfUiImageState, load_wolf_ui_image_state, update_wolf_ui_image};

#[component]
pub fn SettingsImageUpdates() -> Element {
    let mut image_state = use_resource(load_wolf_ui_image_state);
    let progress = use_signal(|| None::<f64>);
    let update_runner = use_action(
        move |image: String, mut progress: Signal<Option<f64>>| async move {
            update_wolf_ui_image(image, move |value| progress.set(Some(value)))
                .await
                .map_err(std::io::Error::other)
        },
    );

    rsx! {
        match &*image_state.read_unchecked() {
            Some(Ok(state)) => rsx! {
                UpdatePanel {
                    state: state.clone(),
                    progress,
                    update_runner,
                    onupdated: move |_| image_state.restart(),
                }
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
fn UpdatePanel(
    state: WolfUiImageState,
    progress: Signal<Option<f64>>,
    mut update_runner: Action<(String, Signal<Option<f64>>), bool>,
    onupdated: EventHandler<()>,
) -> Element {
    let pending = update_runner.pending();
    let progress_value = progress().unwrap_or(0.0).round() as u8;
    let update_result = update_runner.value();
    let update_downloaded = update_result
        .as_ref()
        .and_then(|result| result.as_ref().ok().map(|downloaded| downloaded()));
    let update_failed = matches!(update_result, Some(Err(_)));
    let image_status = image_status(&state, update_downloaded);
    let update_disabled = pending || update_downloaded.is_some();
    let progress_visible = pending || progress().is_some();

    rsx! {
        Card { class: "overflow-hidden rounded-2xl bg-card shadow-black/35",
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
                        value: image_status.to_string(),
                        failed: update_failed,
                    }
                }
                UpdateAction {
                    image: state.source.clone(),
                    disabled: update_disabled,
                    pending,
                    progress,
                    progress_value,
                    progress_visible,
                    update_runner,
                    onupdated,
                }
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
        "rounded-2xl border border-border/80 bg-background/35 p-5",
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
fn UpdateAction(
    image: String,
    disabled: bool,
    pending: bool,
    progress: Signal<Option<f64>>,
    progress_value: u8,
    progress_visible: bool,
    update_runner: Action<(String, Signal<Option<f64>>), bool>,
    onupdated: EventHandler<()>,
) -> Element {
    let toasts = use_toasts();

    rsx! {
        div { class: "flex flex-col gap-4",
            div { class: "flex justify-center sm:justify-end",
                Button {
                    size: ButtonSize::Xl,
                    class: "font-bold",
                    action_label: "Update image",
                    disabled,
                    onclick: move |_| {
                        start_image_update(image.clone(), progress, update_runner, toasts, onupdated);
                    },
                    if pending {
                        Spinner { class: "h-5 w-5" }
                        "Updating"
                    } else {
                        "Update Image"
                    }
                }
            }
            if progress_visible {
                ProgressPanel {
                    label: "Downloading image".to_string(),
                    progress: progress_value,
                }
            }
        }
    }
}

fn image_status(state: &WolfUiImageState, downloaded: Option<bool>) -> &'static str {
    match (state.installed, downloaded) {
        (false, _) => "Missing",
        (true, Some(_)) => "Up to date",
        (true, None) => "Installed",
    }
}

fn start_image_update(
    image: String,
    mut progress: Signal<Option<f64>>,
    mut update_runner: Action<(String, Signal<Option<f64>>), bool>,
    mut toasts: ToastContext,
    onupdated: EventHandler<()>,
) {
    if update_runner.pending() {
        return;
    }

    progress.set(Some(0.0));
    update_runner.reset();

    spawn(async move {
        update_runner.call(image, progress).await;
        let result = update_runner
            .value()
            .unwrap_or_else(|| Err(std::io::Error::other("Image update did not complete.").into()));
        progress.set(None);

        match result {
            Ok(downloaded) if downloaded() => toasts.show("Wolf UI image updated.", None),
            Ok(_) => toasts.show("Wolf UI image is already up to date.", None),
            Err(_) => toasts.show(
                "Wolf UI image update failed. Try again.",
                ToastOptions::error(),
            ),
        }

        onupdated.call(());
    });
}
