use dioxus::prelude::*;

use crate::Route;
use crate::components::primitives::{Button, ButtonSize, ButtonVariant};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsCategory {
    ImageUpdates,
}

impl SettingsCategory {
    fn from_route(route: &Route) -> Self {
        match route {
            Route::Settings {} | Route::SettingsImageUpdates {} => Self::ImageUpdates,
            _ => Self::ImageUpdates,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::ImageUpdates => "Image Updates",
        }
    }

    fn route(self) -> Route {
        match self {
            Self::ImageUpdates => Route::SettingsImageUpdates {},
        }
    }
}

#[component]
pub fn SettingsLayout() -> Element {
    let route = use_route::<Route>();
    let active = SettingsCategory::from_route(&route);

    rsx! {
        div { class: "flex h-full min-h-0 flex-col bg-background px-6 pt-8 text-foreground sm:px-16 sm:pt-12",
            header { class: "shrink-0 pb-6 sm:pb-8",
                h1 { class: "text-5xl font-bold tracking-tight sm:text-6xl md:text-7xl",
                    "Settings"
                }
            }
            div {
                class: "flex min-h-0 flex-1 flex-col gap-4 sm:flex-row sm:gap-6",
                "data-focus-scope": "true",
                "data-focus-region": "main",
                SettingsSidebar { active }
                main { class: "min-h-0 flex-1 overflow-y-auto scroll-py-4 pb-6 scrollbar-hide",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

#[component]
fn SettingsSidebar(active: SettingsCategory) -> Element {
    rsx! {
        nav { class: "shrink-0 rounded-2xl border border-border bg-card p-2 mb-4 shadow-2xl shadow-black/25 sm:w-72",
            SettingsNavItem { category: SettingsCategory::ImageUpdates, active }
        }
    }
}

#[component]
fn SettingsNavItem(category: SettingsCategory, active: SettingsCategory) -> Element {
    let selected = category == active;

    rsx! {
        Button {
            variant: if selected { ButtonVariant::Default } else { ButtonVariant::Outline },
            size: ButtonSize::Xl,
            class: "w-full justify-start",
            to: category.route().to_string(),
            action_label: category.label(),
            autofocus: selected,
            "{category.label()}"
        }
    }
}
