use std::collections::HashMap;

use dioxus::prelude::*;

use crate::components::{AppCardData, AppStatusTone, Spinner};
use crate::domain::app_actions::{ActionStatus, ActionStatusKind};

#[component]
pub fn SelectedAppMeta(
    app: Option<AppCardData>,
    action_statuses: HashMap<String, ActionStatus>,
) -> Element {
    let Some(app) = app else {
        return rsx! {};
    };
    let badge_class = status_badge_class(app.status.tone);
    let action_status = action_statuses.get(&app.id).cloned();

    rsx! {
        div { class: "pointer-events-none flex min-h-36 shrink-0 flex-col items-center justify-start gap-3 px-6 text-center lg:min-h-40",
            h2 { class: "max-w-[80vw] truncate text-3xl font-black tracking-tight md:text-4xl xl:text-5xl 2xl:text-6xl", "{app.title}" }
            div { class: "inline-flex items-center gap-3 rounded-full border px-4 py-2 font-mono text-xs font-bold uppercase tracking-widest shadow-2xl shadow-black/30 {badge_class}",
                span { class: "h-2 w-2 rounded-full bg-current shadow-[0_0_16px_currentColor]" }
                span { "{app.status.label}" }
            }
            ActionProgress { status: action_status }
        }
    }
}

#[component]
fn ActionProgress(status: Option<ActionStatus>) -> Element {
    let Some(status) = status else {
        return rsx! {};
    };
    let status_class = match status.kind {
        ActionStatusKind::Error => "border-red-400/30 bg-red-400/10 text-red-200",
        ActionStatusKind::Success => "border-emerald-400/30 bg-emerald-400/10 text-emerald-200",
        ActionStatusKind::Loading | ActionStatusKind::Progress => {
            "border-yellow-300/30 bg-yellow-300/10 text-yellow-100"
        }
    };
    let progress = status.progress.unwrap_or(0.0).clamp(0.0, 100.0);

    rsx! {
        div { class: "pointer-events-auto min-w-72 max-w-[90vw] rounded-2xl border px-4 py-3 shadow-2xl shadow-black/30 {status_class}", role: "status",
            div { class: "flex items-center justify-center gap-3 text-sm font-semibold",
                if matches!(status.kind, ActionStatusKind::Loading | ActionStatusKind::Progress) {
                    Spinner { class: "h-4 w-4".to_string() }
                }
                span { "{status.message}" }
            }
            if status.progress.is_some() {
                div { class: "mt-2 h-2 overflow-hidden rounded-full bg-black/30",
                    div {
                        class: "h-full rounded-full bg-current transition-all duration-300",
                        style: "width: {progress}%",
                    }
                }
            }
        }
    }
}

fn status_badge_class(tone: AppStatusTone) -> &'static str {
    match tone {
        AppStatusTone::Ready => "border-emerald-400/30 bg-emerald-400/10 text-emerald-300",
        AppStatusTone::Warning => "border-yellow-300/30 bg-yellow-300/10 text-yellow-200",
    }
}
