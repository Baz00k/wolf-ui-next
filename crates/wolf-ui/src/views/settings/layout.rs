use dioxus::prelude::*;

use crate::Route;
use crate::components::primitives::{Button, ButtonSize, ButtonVariant};
use crate::input::navigate_hint;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsCategory {
    Update,
}

impl SettingsCategory {
    fn from_route(route: &Route) -> Self {
        match route {
            Route::Settings {} | Route::SettingsImageUpdates {} => Self::Update,
            _ => Self::Update,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Update => "Update",
        }
    }

    fn route(self) -> Route {
        match self {
            Self::Update => Route::SettingsImageUpdates {},
        }
    }
}

#[component]
pub fn SettingsLayout() -> Element {
    let route = use_route::<Route>();
    let active = SettingsCategory::from_route(&route);

    rsx! {
        div { class: "flex h-full min-h-0 flex-col bg-background px-6 pt-8 text-foreground sm:px-10 sm:pt-10 lg:px-16 lg:pt-12",
            header { class: "shrink-0 pb-6 lg:pb-8",
                h1 { class: "text-5xl font-bold tracking-tight lg:text-6xl 2xl:text-7xl", "Settings" }
            }
            div {
                class: "flex min-h-0 flex-1 flex-col gap-4 lg:flex-row lg:gap-6",
                "data-focus-scope": "true",
                "data-focus-region": "main",
                "data-scope-actions": navigate_hint("Navigate"),
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
        nav { class: "shrink-0 rounded-2xl border border-border bg-card p-2 mb-4 shadow-2xl shadow-black/25 lg:w-72",
            SettingsNavItem { category: SettingsCategory::Update, active }
        }
    }
}

#[component]
fn SettingsNavItem(category: SettingsCategory, active: SettingsCategory) -> Element {
    let selected = category == active;

    rsx! {
        Button {
            variant: if selected { ButtonVariant::MenuActive } else { ButtonVariant::Menu },
            size: ButtonSize::Xl,
            class: "w-full justify-start rounded-xl px-5 font-medium",
            to: category.route().to_string(),
            action_label: category.label(),
            autofocus: selected,
            "{category.label()}"
        }
    }
}
