use dioxus::prelude::*;
use tw_merge::tw_merge;

/// Responsive card grid used by the profile and app browsers.
///
/// `columns` caps the grid width so rows stay centered on ultra-wide screens.
/// With `fit`, empty tracks collapse so a handful of cards centers nicely;
/// without it, track sizes stay stable while content loads in.
#[component]
pub fn CardGrid(
    #[props(default = 5)] columns: usize,
    #[props(default)] fit: bool,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let columns_class = if fit {
        "grid-cols-[repeat(auto-fit,minmax(min(100%,14rem),18rem))] sm:grid-cols-[repeat(auto-fit,minmax(16rem,20rem))] xl:grid-cols-[repeat(auto-fit,minmax(18rem,22rem))]"
    } else {
        "grid-cols-[repeat(auto-fill,minmax(min(100%,14rem),18rem))] sm:grid-cols-[repeat(auto-fill,minmax(16rem,20rem))] xl:grid-cols-[repeat(auto-fill,minmax(18rem,22rem))]"
    };
    let class = tw_merge!(
        "mx-auto grid w-full justify-center gap-4 p-2 sm:gap-5 sm:p-3 xl:gap-6 lg:p-4 2xl:gap-8 2xl:p-5",
        columns_class,
        class,
    );
    let max_width = format!(
        "max-width: min(100%, calc(22rem * {columns} + 2rem * {}));",
        columns.saturating_sub(1)
    );

    rsx! {
        div { class, style: max_width,
            {children}
        }
    }
}

/// Scrollable viewport for a [`CardGrid`] with hidden scrollbars and scroll
/// padding tuned for gamepad navigation.
#[component]
pub fn CardGridViewport(#[props(default)] class: String, children: Element) -> Element {
    let class = tw_merge!(
        "h-full w-full overflow-y-auto overflow-x-hidden scroll-py-6 scrollbar-hide sm:scroll-py-7 lg:scroll-py-8",
        class,
    );

    rsx! {
        div { class,
            {children}
        }
    }
}
