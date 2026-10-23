use std::collections::HashMap;

use dioxus::core::spawn_forever;
use dioxus::prelude::*;

use crate::api::ApiContext;
use crate::components::primitives::{ToastContext, ToastOptions, use_toasts};

/// Why an image is being pulled. Decides the progress label and how the result is announced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PullKind {
    Download,
    Update,
    /// Wolf UI checking its own image at startup. Only a new image is worth interrupting for.
    StartupUpdate,
}

impl PullKind {
    pub(crate) const fn progress_label(self) -> &'static str {
        match self {
            Self::Download => "Downloading",
            Self::Update | Self::StartupUpdate => "Updating",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PullOutcome {
    Downloaded,
    UpToDate,
    Failed,
}

/// Docker image pulls keyed by image reference.
///
/// Pulls run on the root scope so they outlive the dialog or screen that started them, and several
/// images can pull at once. Starting a pull for an image that is already pulling does nothing.
#[derive(Clone, Copy)]
pub(crate) struct ImagePulls {
    /// Changes only when a pull starts or finishes.
    active: Signal<HashMap<String, PullKind>>,
    /// Whole-percent progress per active pull, kept apart so progress only re-renders progress UI.
    progress: Signal<HashMap<String, u8>>,
    /// Latest result per image for the lifetime of the app.
    outcomes: Signal<HashMap<String, PullOutcome>>,
    toasts: ToastContext,
}

/// Provides [`ImagePulls`]. Call after the toast provider.
pub(crate) fn use_image_pulls_provider() {
    let active = use_signal(HashMap::new);
    let progress = use_signal(HashMap::new);
    let outcomes = use_signal(HashMap::new);
    let toasts = use_toasts();
    use_context_provider(|| ImagePulls {
        active,
        progress,
        outcomes,
        toasts,
    });
}

pub(crate) fn use_image_pulls() -> ImagePulls {
    consume_context::<ImagePulls>()
}

impl ImagePulls {
    pub(crate) fn kind(&self, image: &str) -> Option<PullKind> {
        self.active.read().get(image).copied()
    }

    pub(crate) fn progress(&self, image: &str) -> u8 {
        self.progress.read().get(image).copied().unwrap_or(0)
    }

    pub(crate) fn outcome(&self, image: &str) -> Option<PullOutcome> {
        self.outcomes.read().get(image).copied()
    }

    /// Marks images present once a pull of them succeeded, without waiting for the next refresh.
    pub(crate) fn mark_pulled(&self, images: &mut HashMap<String, bool>) {
        mark_pulled(images, &self.outcomes.read());
    }

    /// Starts pulling `image` in the background. `title` names it in the result toast.
    pub(crate) fn start(self, image: String, kind: PullKind, title: String) {
        let Self {
            mut active,
            mut progress,
            mut outcomes,
            mut toasts,
        } = self;

        if active.peek().contains_key(&image) {
            return;
        }

        active.write().insert(image.clone(), kind);
        progress.write().insert(image.clone(), 0);
        let api = ApiContext::consume();

        spawn_forever(async move {
            let result = pull_image(&api, &image, |value| {
                let value = value.round() as u8;
                if progress.peek().get(&image) != Some(&value) {
                    progress.write().insert(image.clone(), value);
                }
            })
            .await;

            let outcome = match result {
                Ok(true) => PullOutcome::Downloaded,
                Ok(false) => PullOutcome::UpToDate,
                Err(error) => {
                    tracing::warn!(image, error, "image pull failed");
                    PullOutcome::Failed
                }
            };

            active.write().remove(&image);
            progress.write().remove(&image);
            outcomes.write().insert(image, outcome);
            announce(&mut toasts, kind, &title, outcome);
        });
    }
}

async fn pull_image(
    api: &wolf_api::WolfApi,
    image: &str,
    on_progress: impl FnMut(f64),
) -> Result<bool, String> {
    let docker = api.docker();
    let downloaded = docker
        .pull_image(image, on_progress)
        .await
        .map_err(|error| error.to_string())?;
    let exists = docker
        .image_exists(image)
        .await
        .map_err(|error| error.to_string())?;
    if !exists {
        return Err("Wolf reported pull success, but the image is still missing.".to_string());
    }
    Ok(downloaded)
}

fn announce(toasts: &mut ToastContext, kind: PullKind, title: &str, outcome: PullOutcome) {
    match (kind, outcome) {
        (PullKind::Download, PullOutcome::Downloaded | PullOutcome::UpToDate) => {
            toasts.show(format!("{title} downloaded."), None);
        }
        (PullKind::Download, PullOutcome::Failed) => toasts.show(
            format!("{title} download failed. Try again."),
            ToastOptions::error(),
        ),
        (PullKind::Update, PullOutcome::Downloaded) => {
            toasts.show(format!("{title} updated."), None);
        }
        (PullKind::Update, PullOutcome::UpToDate) => {
            toasts.show(format!("{title} is up to date."), None);
        }
        (PullKind::Update, PullOutcome::Failed) => toasts.show(
            format!("{title} update failed. Try again."),
            ToastOptions::error(),
        ),
        (PullKind::StartupUpdate, PullOutcome::Downloaded) => toasts.show(
            format!("A new {title} was downloaded. Restart to apply the update."),
            ToastOptions::default().persistent(),
        ),
        (PullKind::StartupUpdate, PullOutcome::UpToDate | PullOutcome::Failed) => {}
    }
}

fn mark_pulled(images: &mut HashMap<String, bool>, outcomes: &HashMap<String, PullOutcome>) {
    for (image, exists) in images.iter_mut() {
        if matches!(
            outcomes.get(image),
            Some(PullOutcome::Downloaded | PullOutcome::UpToDate)
        ) {
            *exists = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_pulls_mark_images_present() {
        let mut images = HashMap::from([
            ("downloaded".to_string(), false),
            ("up-to-date".to_string(), false),
            ("failed".to_string(), false),
            ("untouched".to_string(), false),
        ]);
        let outcomes = HashMap::from([
            ("downloaded".to_string(), PullOutcome::Downloaded),
            ("up-to-date".to_string(), PullOutcome::UpToDate),
            ("failed".to_string(), PullOutcome::Failed),
            ("not-listed".to_string(), PullOutcome::Downloaded),
        ]);

        mark_pulled(&mut images, &outcomes);

        assert!(images["downloaded"]);
        assert!(images["up-to-date"]);
        assert!(!images["failed"]);
        assert!(!images["untouched"]);
        assert!(!images.contains_key("not-listed"));
    }
}
