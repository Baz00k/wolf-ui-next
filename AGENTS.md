# Wolf UI Next

Wolf UI is the launcher and first-screen experience for [Games on Whales](https://games-on-whales.github.io/) when a new Moonlight session starts.

It is the thing users see before they play. If it feels slow, awkward, unreadable, or hostile to their controller, the whole streaming session feels broken before the game even launches.

We are replacing the old Godot-based Wolf UI with a Rust + Dioxus application that feels native, modern, fast, and invisible in the right ways.

## Who we are building for

- People starting a Moonlight session from a TV, handheld, phone, tablet, desktop, browser, or whatever device they happen to have.
- People using an Xbox controller, PlayStation controller, Switch controller, keyboard, mouse, touchscreen, remote, or some messy combination of them.
- People sitting ten feet from the screen who still need to read, navigate, recover from mistakes, and launch a game without thinking about the UI.
- People who do not care that Wolf UI exists. They want to pick up a controller and play.
- Admins who need Wolf UI to be reliable and cheap to keep running because every active session gets one.

The player comes first. The admin comes second. The implementation comes after both.

## Product goal

Make session startup feel seamless.

Wolf UI should get out of the user's way as quickly as possible while still being clear, beautiful, and forgiving.
The best version of this app makes the user feel like the device, controller, stream, and game library are one coherent system.

This is not a generic desktop app. It is a 10-foot, responsive, input-agnostic game launcher for streamed games.

## Design principles

### It must feel native everywhere

Do not design only for the machine you are sitting at. A change is not good enough unless it still makes sense on TVs, phones, desktops, handhelds, touchscreens, and controllers.

Native does not mean copying platform chrome. It means respecting the user's current device, input method, distance from the screen, and expectations.

### Every input path is first-class

Gamepad, keyboard, mouse, touch, and remote navigation must stay coherent with each other. Focus state is product state. Hover-only, mouse-only, and keyboard-only affordances are bugs unless there is an equivalent path for the other inputs.

A user should be able to pick up any controller and continue without re-learning the screen.

### Readability beats density

This is a 10-foot UI. Prefer clear hierarchy, strong contrast, predictable spacing, and obvious focus over compact information. If a screen only works at monitor distance, it does not work.

### Fast is a feature

The launcher must feel instant. Avoid unnecessary loading, animation, layout instability, blocking work, allocation, cloning, polling, and background activity. Wolf UI may keep running even when it is not visible; idle resource usage matters.

### The UI should disappear

Do not add ceremony. Do not make users manage the launcher. Help them recognize the game they want, launch it, and recover cleanly if something goes wrong.

### Prefer obvious behavior

The best interaction is the one users predict without reading. If two implementations are technically valid, choose the one that makes the product model simpler.

## Engineering posture

- Use Rust as the backbone, not as decoration. Model state and transitions so invalid UI states are hard to represent.
- Keep Dioxus details local to UI boundaries where possible. Product behavior should be understandable without reading framework trivia.
- Reuse existing patterns before inventing new ones. Parallel focus systems, styling conventions, or API shapes are bugs.
- Delete code that is no longer pulling its weight.
- Prefer boring, explicit code over clever abstractions. Add abstraction only when it removes real duplication in product concepts.
- Avoid hidden work. Background tasks, timers, resources, and subscriptions must have a reason to exist and a clear lifetime.
- Treat performance as UX. Do not allocate, clone, parse, fetch, or re-render when a smaller change would do.
- Test behavior that can break: navigation, focus movement, input parity, loading/error states, and state transitions.

## How agents should think

Before changing code, ask:

- Does this make launching a game faster, clearer, or more reliable?
- Does it work with gamepad, keyboard, mouse, touch, and remote-style navigation?
- Does it remain readable at TV distance and usable on small screens?
- Does it reduce or preserve resource usage while idle?
- Does it fit the existing mental model of the app?

If the answer is no, push back or propose a better route. Do not be afraid to challenge the user when a requested implementation would make the product worse.
Be concrete: explain what breaks and offer the simpler product-shaped solution.

## Runtime

This app will always run in a Docker container. This is the only supported environment.
Do not add any code that adds compatibility with other runtimes.

## Useful references

- Dioxus documentation: https://dioxuslabs.com/learn/0.7/
- Games on Whales / Wolf documentation: https://games-on-whales.github.io/wolf/stable/
