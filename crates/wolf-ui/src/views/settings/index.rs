use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn Settings() -> Element {
    let navigator = use_navigator();

    use_effect(move || {
        navigator.replace(Route::SettingsImageUpdates {});
    });

    rsx! {}
}
