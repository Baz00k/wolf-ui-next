use dioxus::prelude::*;
use tw_merge::tw_merge;

use crate::components::primitives::Focusable;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Solid primary action.
    #[default]
    Default,
    Secondary,
    Outline,
    Destructive,
    Ghost,
}

impl ButtonVariant {
    fn class(self) -> &'static str {
        match self {
            ButtonVariant::Default => "bg-primary text-primary-foreground hover:bg-primary/90",
            ButtonVariant::Secondary => {
                "bg-secondary text-secondary-foreground hover:bg-secondary/80"
            }
            ButtonVariant::Outline => {
                "border border-input bg-background hover:bg-accent hover:text-accent-foreground"
            }
            ButtonVariant::Destructive => {
                "bg-destructive text-destructive-foreground hover:bg-destructive/90"
            }
            ButtonVariant::Ghost => {
                "hover:border-foreground/30 hover:bg-accent hover:text-accent-foreground focus-visible:border-foreground focus-visible:bg-accent focus-visible:text-accent-foreground"
            }
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
    Xl,
    IconSm,
    Icon,
    IconLg,
}

impl ButtonSize {
    fn class(self) -> &'static str {
        match self {
            ButtonSize::Default => "h-12 px-6 py-3 text-base sm:h-14",
            ButtonSize::Sm => "h-10 rounded-lg px-4 py-2 text-base sm:h-12",
            ButtonSize::Lg => "h-14 rounded-xl px-8 py-4 text-lg sm:h-16",
            ButtonSize::Xl => "h-16 rounded-xl px-8 py-4 text-lg font-semibold sm:h-18",
            ButtonSize::IconSm => "h-10 w-10 rounded-full text-base sm:h-12 sm:w-12",
            ButtonSize::Icon => "h-12 w-12 text-base sm:h-14 sm:w-14",
            ButtonSize::IconLg => "h-14 w-14 text-lg sm:h-16 sm:w-16",
        }
    }
}

#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
    #[props(default)] class: String,
    #[props(default)] to: Option<String>,
    #[props(default = "Select".to_string())] action_label: String,
    #[props(default)] autofocus: bool,
    #[props(default)] disabled: bool,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let class = tw_merge!(
        "inline-flex items-center justify-center gap-3 whitespace-nowrap rounded-lg border border-transparent font-medium leading-none outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring/80 focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-50",
        variant.class(),
        size.class(),
        class,
    );

    rsx! {
        Focusable {
            class,
            to,
            action_label,
            autofocus,
            disabled,
            onclick,
            {children}
        }
    }
}
