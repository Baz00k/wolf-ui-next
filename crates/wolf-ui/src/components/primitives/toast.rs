use std::time::Duration;

use dioxus::prelude::*;
use tokio::time::sleep;

const DEFAULT_AUTO_HIDE: Duration = Duration::from_secs(5);

#[derive(Clone, Copy)]
pub struct ToastContext {
    messages: Signal<Vec<ToastMessage>>,
    next_id: Signal<u64>,
}

impl ToastContext {
    pub fn show(&mut self, message: impl Into<String>, options: impl Into<Option<ToastOptions>>) {
        let options = options.into().unwrap_or_default();
        let id = self.next_message_id();
        self.messages.write().push(ToastMessage {
            id,
            message: message.into(),
            variant: options.variant,
            auto_hide: options.auto_hide,
        });
    }

    pub fn dismiss(&mut self, id: u64) {
        self.messages.write().retain(|message| message.id != id);
    }

    fn next_message_id(&mut self) -> u64 {
        let id = (self.next_id)();
        self.next_id.set(id + 1);
        id
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ToastOptions {
    pub variant: ToastVariant,
    pub auto_hide: Option<Duration>,
}

impl ToastOptions {
    pub fn error() -> Self {
        Self {
            variant: ToastVariant::Error,
            ..Self::default()
        }
    }

    pub fn persistent(mut self) -> Self {
        self.auto_hide = None;
        self
    }
}

impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            variant: ToastVariant::Info,
            auto_hide: Some(DEFAULT_AUTO_HIDE),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ToastMessage {
    id: u64,
    message: String,
    variant: ToastVariant,
    auto_hide: Option<Duration>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    Info,
    Error,
}

pub fn use_toast_provider() {
    let messages = use_signal(Vec::<ToastMessage>::new);
    let next_id = use_signal(|| 1_u64);
    use_context_provider(|| ToastContext { messages, next_id });
}

pub fn use_toasts() -> ToastContext {
    consume_context::<ToastContext>()
}

#[component]
pub fn ToastViewport() -> Element {
    let toasts = use_toasts();
    let messages = (toasts.messages)();

    rsx! {
        div { class: "pointer-events-none fixed inset-x-4 bottom-16 z-1000 flex flex-col-reverse items-center gap-3",
            for message in messages {
                Toast {
                    key: "{message.id}",
                    id: message.id,
                    message: message.message.clone(),
                    variant: message.variant,
                    auto_hide: message.auto_hide,
                    ondismiss: move |_| {
                        let mut toasts = toasts;
                        toasts.dismiss(message.id);
                    },
                }
            }
        }
    }
}

#[component]
pub fn Toast(
    id: u64,
    message: String,
    #[props(default = ToastVariant::Info)] variant: ToastVariant,
    #[props(default = Some(DEFAULT_AUTO_HIDE))] auto_hide: Option<Duration>,
    ondismiss: EventHandler<()>,
) -> Element {
    use_effect(move || {
        if let Some(delay) = auto_hide {
            spawn(async move {
                sleep(delay).await;
                ondismiss.call(());
            });
        }
    });

    let tone = match variant {
        ToastVariant::Info => "border-border bg-card/95 text-card-foreground",
        ToastVariant::Error => "border-destructive bg-destructive/30 text-destructive-foreground",
    };

    rsx! {
        div {
            key: "{id}",
            class: "toast-enter flex w-[min(28rem,calc(100vw-2rem))] items-center gap-3 overflow-hidden rounded-2xl border px-4 py-3 shadow-2xl shadow-black/40 backdrop-blur backdrop-brightness-75 {tone}",
            p { class: "min-w-0 flex-1 text-sm font-semibold leading-6", "{message}" }
        }
    }
}
