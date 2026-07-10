use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::input::{UiAction, UiHint};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct ActionHint {
    pub action: UiHint,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handler: Option<String>,
}

impl ActionHint {
    pub fn new(action: UiHint, label: impl Into<String>) -> Self {
        Self {
            action,
            label: label.into(),
            handler: None,
        }
    }
}

#[derive(Clone)]
pub(super) struct ActionRegistry {
    next_id: Rc<Cell<u64>>,
    handlers: Rc<RefCell<HashMap<String, Callback<()>>>>,
}

impl ActionRegistry {
    pub(super) fn new() -> Self {
        Self {
            next_id: Rc::new(Cell::new(1)),
            handlers: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn register(&self, handler: Callback<()>) -> String {
        let id = format!("action-{}", self.next_id.get());
        self.next_id.set(self.next_id.get() + 1);
        self.handlers.borrow_mut().insert(id.clone(), handler);
        id
    }

    fn unregister(&self, id: &str) {
        self.handlers.borrow_mut().remove(id);
    }

    pub(super) fn call(&self, id: &str) -> bool {
        let Some(handler) = self.handlers.borrow().get(id).copied() else {
            return false;
        };

        handler.call(());
        true
    }
}

pub fn action_hints(hints: impl IntoIterator<Item = ActionHint>) -> String {
    serde_json::to_string(&hints.into_iter().collect::<Vec<_>>()).unwrap_or_else(|error| {
        tracing::warn!(target: "wolf-ui-input", "failed to serialize action hints: {error}");
        "[]".to_string()
    })
}

pub fn native_action(action: UiAction, label: impl Into<String>) -> String {
    action_hints([ActionHint::new(UiHint::from(action), label)])
}

pub fn use_ui_action(
    action: UiAction,
    label: impl Into<String>,
    handler: impl FnMut() + 'static,
) -> String {
    action_hints([use_ui_action_hint(action, label, handler)])
}

pub fn use_ui_action_hint(
    action: UiAction,
    label: impl Into<String>,
    mut handler: impl FnMut() + 'static,
) -> ActionHint {
    let registry = use_context::<ActionRegistry>();
    let callback = use_callback(move |()| handler());
    let handler_id = use_hook({
        let registry = registry.clone();
        move || registry.register(callback)
    });
    use_drop({
        let registry = registry.clone();
        let handler_id = handler_id.clone();
        move || registry.unregister(&handler_id)
    });

    ActionHint {
        action: UiHint::from(action),
        label: label.into(),
        handler: Some(handler_id),
    }
}

pub(super) fn use_action_bridge(registry: ActionRegistry) {
    use_effect(move || {
        let registry = registry.clone();
        spawn(async move {
            let mut eval = document::eval(
                r#"
                document.addEventListener("wolf-ui-action", (event) => {
                    dioxus.send(event.detail?.handler ?? null);
                });
                "#,
            );

            while let Ok(handler_id) = eval.recv::<String>().await {
                registry.call(&handler_id);
            }
        });
    });
}
