use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ButtonVariant {
    Default,
    Secondary,
    Outline,
    Ghost,
    Destructive,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ButtonSize {
    Default,
    Sm,
    Lg,
    Icon,
}

#[component]
pub fn Button(
    #[props(default = ButtonVariant::Default)] variant: ButtonVariant,
    #[props(default = ButtonSize::Default)] size: ButtonSize,
    #[props(default)] class: String,
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
    };
    let size_class = match size {
        ButtonSize::Default => "h-10 px-4 py-2",
        ButtonSize::Sm => "h-9 rounded-md px-3",
        ButtonSize::Lg => "h-11 rounded-xl px-8",
        ButtonSize::Icon => "h-10 w-10",
    };

    rsx! {
        button {
            class: "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-lg text-sm font-medium outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 {variant_class} {size_class} {class}",
            onclick: move |event| {
                if let Some(handler) = onclick {
                    handler.call(event);
                }
            },
            {children}
        }
    }
}
