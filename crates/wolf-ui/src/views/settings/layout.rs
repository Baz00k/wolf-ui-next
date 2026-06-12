use dioxus::prelude::*;
use tw_merge::tw_merge;

use crate::Route;
use crate::input::{UiAction, native_action, navigate_hint};

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
                main { class: "min-h-0 flex-1 overflow-y-auto pb-6 scrollbar-hide",
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
    let class = tw_merge!(
        "flex h-14 w-full items-center rounded-xl border px-5 text-left font-medium outline-none focus:ring-2 focus:ring-ring/50",
        if selected {
            "border-primary/40 bg-primary/15 text-foreground shadow-lg shadow-primary/10 focus:border-primary focus:bg-primary/20"
        } else {
            "border-transparent text-muted-foreground hover:border-foreground/30 hover:bg-accent hover:text-accent-foreground focus:border-foreground focus:bg-accent focus:text-accent-foreground"
        },
    );
    let actions = native_action(UiAction::Accept, category.label());

    rsx! {
        Link {
            to: category.route(),
            class,
            "data-focusable": "true",
            "data-actions": actions,
            "{category.label()}"
        }
    }
}
