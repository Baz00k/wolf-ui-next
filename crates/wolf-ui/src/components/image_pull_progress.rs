use dioxus::prelude::*;

use crate::components::primitives::{ProgressPanel, ProgressTone};
use crate::domain::image_pulls::{PullKind, use_image_pulls};

/// Labelled progress for an active image pull. Only this component re-renders as progress moves.
#[component]
pub fn ImagePullProgress(image: String, kind: PullKind) -> Element {
    let progress = use_image_pulls().progress(&image);

    rsx! {
        ProgressPanel {
            label: kind.progress_label(),
            progress,
            tone: ProgressTone::Warning,
        }
    }
}
