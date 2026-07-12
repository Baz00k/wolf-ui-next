# Focus Navigation

`assets/focus-navigation.js` is the browser-side focus engine for controller,
allowing for UI navigation using gamepad, keyboard, or remote.

## Quick Start

For standard interactive elements, use the `Focusable` component:

```rust
rsx! {
    Focusable {
        action_label: "Launch",
        autofocus: true,
        onclick: move |_| launch_app(),
        "Play"
    }
}
```

This is equivalent to rendering an interactive element with:

- `data-focusable="true"`
- `data-actions` containing an accept action hint
- `data-autofocus="true"` when requested

## Focusable Elements

An element participates in directional focus when it has:

```rust
"data-focusable": "true"
```

The element is ignored when it is hidden, zero-sized, `disabled`,
`aria-disabled="true"`, or `inert`.

Manual example:

```rust
button {
    "data-focusable": "true",
    "data-actions": native_action(UiAction::Accept, "Open"),
    onclick: move |_| open(),
    "Open"
}
```

## Focus Scopes

`data-focus-scope` limits directional navigation to a section of the page.
Without a scope, navigation falls back to the first scope on the page, then the
document body.

```rust
section {
    "data-focus-scope": "true",
    "data-scope-actions": action_hints([ActionHint::new(UiHint::Navigate, "Navigate")]),
    Focusable { action_label: "Open", "First" }
    Focusable { action_label: "Open", "Second" }
}
```

Use a scope for a coherent navigation area, such as a grid, top bar, sidebar, or
dialog.

Only focusable descendants inside the active scope are considered for normal
directional movement.

## Focus Regions

`data-focus-region` gives the focus engine a fallback order between areas. When
no good geometric candidate exists inside the current scope, the engine searches
visible regions in DOM order.

```rust
main {
    "data-focus-root": "true",
    nav {
        "data-focus-scope": "true",
        "data-focus-region": "top-bar",
        "data-scope-actions": action_hints([ActionHint::new(UiHint::Navigate, "Navigate")]),
        Focusable { "Profiles" }
    }
    section {
        "data-focus-scope": "true",
        "data-focus-region": "main",
        "data-scope-actions": action_hints([ActionHint::new(UiHint::Navigate, "Navigate")]),
        Focusable { "Game 1" }
    }
}
```

Use `data-focus-root="true"` around related regions so fallback navigation does
not escape to unrelated page content.

Regions are useful when a page has multiple focus scopes that should still feel
connected, such as a top bar above a grid. They are not needed for a simple list
or a single grid.

## Focus Traps

`data-focus-trap="true"` makes the visible trap the active scope. Focus entering
outside the trap is redirected to the first focusable element inside it.

Dialogs already do this through the `Dialog` primitive:

```rust
Dialog {
    scope_actions: native_action(UiAction::Cancel, "Close"),
    Focusable { autofocus: true, action_label: "Confirm", "Confirm" }
}
```

Use a trap only for modal UI. Do not trap focus for normal page sections.

When more than one visible trap exists, the last visible trap in DOM order is the
active trap.

## Autofocus

`data-autofocus="true"` marks a focusable element as the preferred initial focus
target.

```rust
Focusable {
    autofocus: true,
    action_label: "Continue",
    "Continue"
}
```

If a focus trap is active, autofocus is resolved inside that trap. Otherwise it
is resolved across the document. An autofocus target may replace focus in a
different focus scope, which allows asynchronously loaded page content to take
initial focus from temporary header controls. Existing focus inside the target's
own scope is preserved.

## Actions

Actions drive the footer glyphs and optional Rust action handlers.

Use `data-actions` to describe actions that apply to the currently focused element:

```rust
Focusable {
    action_label: "Launch",
    onclick: move |_| launch(),
    "Launch game"
}
```

Use `data-scope-actions` to describe actions that apply while focus is inside that scope:

```rust
div {
    "data-focus-scope": "true",
    "data-scope-actions": action_hints([
        ActionHint::new(UiHint::Navigate, "Move"),
        ActionHint::new(UiHint::PageDown, "Next page"),
    ]),
    // focusable children
}
```

Hints closest to the focused element override broader hints with the same action.
The merge order is root scope, current scope, then focused element.

### Helpers (`crate::input`)

`native_action(action, label)` produces a hint string for actions that the
focus engine handles natively, such as activating a link or a button. No Rust
handler is needed:

```rust
// Focusable does this automatically, but for manual elements:
"data-actions": native_action(UiAction::Accept, "Open")
```

`action_hints([...])` combines several `ActionHint` values into one string. Use
it when a scope or element needs more than one footer glyph:

```rust
"data-scope-actions": action_hints([
    ActionHint::new(UiHint::PageDown, "Next page"),
    ActionHint::new(UiHint::Cancel, "Close"),
])
```

`use_ui_action(action, label, handler)` registers a Rust callback that fires
when the action is triggered. Use it when the action needs to run Rust logic:

```rust
let close_action = use_ui_action(UiCommand::Cancel, "Close", move || onclose.call(()));

rsx! {
    Dialog { scope_actions: close_action,
        Focusable { autofocus: true, "Keep playing" }
    }
}
```

### Footer grouping

Directional actions (`Left`, `Right`, `Up`, `Down`) display as one `navigate`
footer hint. `Accept`, `Cancel`, `Menu`, `PageUp`, and `PageDown` each have
their own footer hint. Page actions are keyboard-only and scroll the nearest
scrollable container; they are hidden from gamepad hints.

## Navigation Rules

Directional navigation chooses the visible focusable element with the best score
in the requested direction.

- Horizontal movement prefers smaller horizontal distance and penalizes vertical
  drift.
- Vertical movement prefers smaller vertical distance and penalizes horizontal
  drift.
- Candidates must be meaningfully left, right, above, or below the current
  element.
- Normal movement rejects strongly perpendicular candidates. Region fallback
  uses a wider angle so explicitly ordered regions remain connected.
- If no element is focused, navigation focuses the first visible candidate in the
  active scope.

## Practical Guidelines

- Use `Focusable` first.
- Add `data-focus-scope` to each major navigation area.
- Add `data-scope-actions` with a navigate hint when a scope contains
  directional items.
- Use `data-focus-region` for page-level fallback between top bars, grids, and
  side panels.
- Use `data-focus-trap` only for modal UI.
- Keep focusable elements visible, sized, and not disabled when they should be
  reachable.
- Prefer Rust action helpers over hand-written JSON.

## Controller Button Layouts

Gamepad actions use the normalized button values reported by `gilrs`: `South`
accepts and `East` cancels. Wolf UI does not swap these actions based on the
detected controller family; family detection only changes the footer labels.
