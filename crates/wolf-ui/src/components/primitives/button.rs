use dioxus::prelude::*;
use tw_merge::tw_merge;

use crate::components::primitives::Focusable;

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ButtonVariant {
    /// Solid primary action.
    Default,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    /// Menu/list row: transparent until hovered or focused.
    Menu,
    /// Selected menu/list row.
    MenuActive,
    /// Floating chrome control (header icon buttons).
    Chrome,
    /// Floating chrome control with destructive intent.
    ChromeDestructive,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ButtonSize {
    Default,
    Sm,
    Lg,
    Xl,
    IconSm,
    Icon,
    IconLg,
}

#[component]
pub fn Button(
    #[props(default = ButtonVariant::Default)] variant: ButtonVariant,
    #[props(default = ButtonSize::Default)] size: ButtonSize,
    #[props(default)] class: String,
    #[props(default)] to: Option<String>,
    #[props(default = "Select".to_string())] action_label: String,
    #[props(default)] autofocus: bool,
    #[props(default)] disabled: bool,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let variant_class = match variant {
        ButtonVariant::Default => "bg-primary text-primary-foreground hover:bg-primary/90",
        ButtonVariant::Secondary => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ButtonVariant::Outline => {
            "border border-input bg-background hover:bg-accent hover:text-accent-foreground"
        }
        ButtonVariant::Ghost => "hover:bg-accent hover:text-accent-foreground",
        ButtonVariant::Destructive => {
            "bg-destructive text-destructive-foreground hover:bg-destructive/90"
        }
        ButtonVariant::Menu => {
            "border border-transparent text-left hover:border-foreground/30 hover:bg-accent hover:text-accent-foreground focus-visible:border-foreground focus-visible:bg-accent focus-visible:text-accent-foreground"
        }
        ButtonVariant::MenuActive => {
            "border border-primary/40 bg-primary/15 text-left text-foreground shadow-lg shadow-primary/10 hover:bg-primary/20 focus-visible:border-primary focus-visible:bg-primary/20"
        }
        ButtonVariant::Chrome => {
            "border border-border/70 bg-card/70 text-muted-foreground shadow-lg shadow-black/20 backdrop-blur hover:border-primary/50 hover:bg-primary/15 hover:text-foreground focus-visible:border-primary/70 focus-visible:bg-primary/15 focus-visible:text-foreground"
        }
        ButtonVariant::ChromeDestructive => {
            "border border-border/70 bg-card/70 text-muted-foreground shadow-lg shadow-black/20 backdrop-blur hover:border-destructive/60 hover:bg-destructive/15 hover:text-destructive-foreground focus-visible:border-destructive/70 focus-visible:bg-destructive/15 focus-visible:text-destructive-foreground"
        }
    };
    let size_class = match size {
        ButtonSize::Default => "h-10 px-4 py-2",
        ButtonSize::Sm => "h-9 rounded-md px-3",
        ButtonSize::Lg => "h-11 rounded-xl px-8",
        ButtonSize::Xl => "h-14 rounded-xl px-8 text-base font-semibold",
        ButtonSize::IconSm => "h-9 w-9 rounded-full",
        ButtonSize::Icon => "h-10 w-10",
        ButtonSize::IconLg => "h-10 w-10 sm:h-12 sm:w-12",
    };
    let class = tw_merge!(
        "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-lg text-sm font-medium outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring/80 focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-50",
        variant_class,
        size_class,
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
