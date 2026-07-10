(() => {
    if (window.__wolfUiFocusInstalled) return;
    window.__wolfUiFocusInstalled = true;

    const FOCUSABLE_SELECTOR = '[data-focusable="true"]';
    const ACTIONS_ATTRIBUTE = "data-actions";
    const SCOPE_ACTIONS_ATTRIBUTE = "data-scope-actions";
    const FOCUS_REGION_ATTRIBUTE = "data-focus-region";
    const MAX_NAVIGATION_ANGLE_RADIANS = (60 * Math.PI) / 180;
    const MAX_REGION_FALLBACK_ANGLE_RADIANS = (85 * Math.PI) / 180;
    const ACTION_TO_HINT = {
        accept: "accept",
        cancel: "cancel",
        menu: "menu",
        left: "navigate",
        right: "navigate",
        up: "navigate",
        down: "navigate",
        "page-up": "page-up",
        "page-down": "page-down",
    };
    let lastActionHintsJson = null;
    const uiSounds = window.__wolfUiSounds ?? { play() {} };

    const AUTOFOCUS_ATTRIBUTE = "data-autofocus";
    let usingMouse = false;
    document.addEventListener("pointerdown", () => (usingMouse = true), true);

    function isVisible(element) {
        return visibleRect(element) !== null;
    }

    function visibleRect(element) {
        const style = window.getComputedStyle(element);
        if (style.display === "none" || style.visibility === "hidden") return null;
        if (element.matches(':disabled,[aria-disabled="true"],[inert]')) return null;
        const rect = element.getBoundingClientRect();
        return rect.width > 0 && rect.height > 0 ? rect : null;
    }

    function scopeFor(element) {
        const trap = activeFocusTrap();
        if (trap) return trap;

        return (
            element?.closest?.("[data-focus-scope]") ?? document.querySelector("[data-focus-scope]") ?? document.body
        );
    }

    function activeFocusTrap() {
        const traps = Array.from(document.querySelectorAll("[data-focus-trap='true']")).filter(isVisible);
        return traps.at(-1) ?? null;
    }

    function candidatesIn(scope) {
        return Array.from(scope.querySelectorAll(FOCUSABLE_SELECTOR)).filter(isVisible);
    }

    function candidateEntriesIn(scope) {
        const entries = [];
        for (const element of scope.querySelectorAll(FOCUSABLE_SELECTOR)) {
            const rect = visibleRect(element);
            if (rect) entries.push({ element, rect });
        }
        return entries;
    }

    function rootScopeFor(element) {
        return element?.closest?.("[data-focus-root]") ?? document.querySelector("[data-focus-root]") ?? document.body;
    }

    function regionFor(element) {
        return element?.closest?.(`[${FOCUS_REGION_ATTRIBUTE}]`) ?? null;
    }

    function orderedRegions(root) {
        return Array.from(root.querySelectorAll(`[${FOCUS_REGION_ATTRIBUTE}]`)).filter(isVisible);
    }

    function fallbackCandidate(action, active, currentRect = active.getBoundingClientRect()) {
        const currentRegion = regionFor(active);
        if (!currentRegion) return null;

        const regions = orderedRegions(rootScopeFor(currentRegion));
        const currentIndex = regions.indexOf(currentRegion);
        if (currentIndex === -1) return null;

        const step = action === "up" || action === "left" ? -1 : action === "down" || action === "right" ? 1 : 0;
        if (step === 0) return null;

        for (let index = currentIndex + step; index >= 0 && index < regions.length; index += step) {
            let best = null;
            let bestScore = Infinity;
            for (const candidate of candidateEntriesIn(regions[index])) {
                const score = directionScore(action, currentRect, candidate.rect, {
                    maxAngle: MAX_REGION_FALLBACK_ANGLE_RADIANS,
                });
                if (score !== null && score < bestScore) {
                    best = candidate.element;
                    bestScore = score;
                }
            }
            if (best) return best;
        }

        return null;
    }

    function center(rect) {
        return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
    }

    function directionVector(action) {
        switch (action) {
            case "left":
                return { x: -1, y: 0 };
            case "right":
                return { x: 1, y: 0 };
            case "up":
                return { x: 0, y: -1 };
            case "down":
                return { x: 0, y: 1 };
            default:
                return null;
        }
    }

    function directionDelta(action, current, candidate) {
        const currentCenter = center(current);
        const candidateCenter = center(candidate);
        const dx = candidateCenter.x - currentCenter.x;
        const dy = candidateCenter.y - currentCenter.y;
        const minHorizontalStep = Math.min(current.width, candidate.width) * 0.25;
        const minVerticalStep = Math.min(current.height, candidate.height) * 0.25;

        switch (action) {
            case "left":
                return dx < -minHorizontalStep ? { dx, dy } : null;
            case "right":
                return dx > minHorizontalStep ? { dx, dy } : null;
            case "up":
                return dy < -minVerticalStep ? { dx, dy } : null;
            case "down":
                return dy > minVerticalStep ? { dx, dy } : null;
            default:
                return null;
        }
    }

    function angleFromDirection(action, dx, dy) {
        const direction = directionVector(action);
        if (!direction) return Infinity;

        const length = Math.hypot(dx, dy);
        if (length === 0) return Infinity;

        const cosine = Math.max(-1, Math.min(1, (dx * direction.x + dy * direction.y) / length));
        return Math.acos(cosine);
    }

    function isWithinNavigationAngle(action, dx, dy, maxAngle) {
        return angleFromDirection(action, dx, dy) <= maxAngle;
    }

    function directionScore(action, current, candidate, options = {}) {
        const delta = directionDelta(action, current, candidate);
        if (!delta) return null;

        if (!isWithinNavigationAngle(action, delta.dx, delta.dy, options.maxAngle)) {
            return null;
        }

        switch (action) {
            case "left":
            case "right":
                return Math.abs(delta.dx) + Math.abs(delta.dy) * 2;
            case "up":
            case "down":
                return Math.abs(delta.dy) + Math.abs(delta.dx) * 2;
            default:
                return null;
        }
    }

    function focusAndScrollElement(element, options = {}) {
        if (!element) return false;

        // Gamepad input needs an explicit focus-visible hint.
        element.focus({ preventScroll: true, focusVisible: !usingMouse });
        window.__wolfUiScrollIntoView?.(element, options);
        return true;
    }

    window.__wolfUiFocusElement = focusAndScrollElement;
    window.__wolfUiFocusSelector = (selector, options = {}) => {
        const element = document.querySelector(selector);
        return focusAndScrollElement(element, options);
    };

    window.__wolfUiFocusAutofocus = () => {
        const scope = activeFocusTrap() ?? document;
        const candidates = Array.from(scope.querySelectorAll(`${FOCUSABLE_SELECTOR}[${AUTOFOCUS_ATTRIBUTE}='true']`));
        const element = candidates.find(isVisible);
        return focusAndScrollElement(element);
    };

    function focusFirst(scope) {
        const first = candidatesIn(scope)[0];
        if (!first) return false;
        focusAndScrollElement(first);
        dispatchActionHintsChanged(first);
        return true;
    }

    function ensureFocusableActiveElement() {
        const active = document.activeElement;
        const trap = activeFocusTrap();
        if (
            active?.matches?.(FOCUSABLE_SELECTOR) &&
            isVisible(active) &&
            (!trap || trap.contains(active))
        ) {
            return true;
        }

        if (focusFirst(scopeFor(active))) return true;
        if (trap) return false;
        return focusFirst(document);
    }

    window.__wolfUiEnsureFocusableActiveElement = ensureFocusableActiveElement;

    const dialogOpeners = [];

    window.__wolfUiCaptureDialogOpener = (dialogId) => {
        const element = document.activeElement?.matches?.(FOCUSABLE_SELECTOR) ? document.activeElement : null;
        dialogOpeners.push({ dialogId, element });
    };

    window.__wolfUiRestoreDialogOpener = (dialogId) => {
        const index = dialogOpeners.findIndex((entry) => entry.dialogId === dialogId);
        if (index === -1) return;

        const [{ element }] = dialogOpeners.splice(index, 1);
        requestAnimationFrame(() =>
            requestAnimationFrame(() => {
                const trap = activeFocusTrap();
                if (element?.isConnected && (!trap || trap.contains(element)) && isVisible(element)) {
                    if (focusAndScrollElement(element, { inline: "nearest" })) return;
                }

                ensureFocusableActiveElement();
            }),
        );
    };

    function moveFocus(action) {
        const active = document.activeElement?.matches?.(FOCUSABLE_SELECTOR) ? document.activeElement : null;
        const scope = scopeFor(active);

        if (!active) return focusFirst(scope);

        const currentRect = active.getBoundingClientRect();
        let best = null;
        let bestScore = Infinity;

        for (const candidate of candidateEntriesIn(scope)) {
            if (candidate.element === active) continue;
            const score = directionScore(action, currentRect, candidate.rect, {
                maxAngle: MAX_NAVIGATION_ANGLE_RADIANS,
            });
            if (score !== null && score < bestScore) {
                best = candidate.element;
                bestScore = score;
            }
        }

        if (!best) {
            best = fallbackCandidate(action, active, currentRect);
            if (!best) return false;
        }

        focusAndScrollElement(best);
        dispatchActionHintsChanged(best);
        return true;
    }

    function actionHintsFor(element) {
        const scope = scopeFor(element);
        const rootScope = rootScopeFor(scope);
        const hints = new Map();

        if (rootScope !== scope) {
            collectActionHints(rootScope, hints, SCOPE_ACTIONS_ATTRIBUTE);
        }
        collectActionHints(scope, hints, SCOPE_ACTIONS_ATTRIBUTE);
        if (element && element !== scope) {
            collectActionHints(element, hints, ACTIONS_ATTRIBUTE);
        }

        return Array.from(hints.values());
    }

    function collectActionHints(element, hints, attributeName) {
        const encodedHints = element?.getAttribute?.(attributeName);
        if (!encodedHints) return;

        let parsedHints = [];
        try {
            parsedHints = JSON.parse(encodedHints);
        } catch (error) {
            console.warn(`Invalid ${attributeName}`, error);
            return;
        }

        if (!Array.isArray(parsedHints)) return;

        for (const hint of parsedHints) {
            if (!hint?.action || !hint?.label) continue;
            hints.set(hint.action, hint);
        }
    }

    function activeAction(action) {
        const hintAction = ACTION_TO_HINT[action] ?? action;
        return actionHintsFor(document.activeElement).find((hint) => hint.action === hintAction) ?? null;
    }

    function dispatchRustAction(action) {
        const hint = activeAction(action);
        if (!hint?.handler) return false;
        document.dispatchEvent(new CustomEvent("wolf-ui-action", { detail: hint }));
        return true;
    }

    function dispatchActionHintsChanged(element = document.activeElement) {
        const hints = actionHintsFor(element?.matches?.(FOCUSABLE_SELECTOR) ? element : null);
        const nextActionHintsJson = JSON.stringify(hints);
        if (nextActionHintsJson === lastActionHintsJson) return;

        lastActionHintsJson = nextActionHintsJson;
        document.dispatchEvent(
            new CustomEvent("wolf-ui-action-hints-changed", {
                detail: hints,
            }),
        );
    }

    document.addEventListener("focusin", (event) => {
        const trap = activeFocusTrap();
        if (trap && !trap.contains(event.target)) {
            focusFirst(trap);
            return;
        }

        dispatchActionHintsChanged(event.target);
    });

    document.addEventListener("focusout", () => {
        queueMicrotask(() => dispatchActionHintsChanged(document.activeElement));
    });

    new MutationObserver(() => {
        queueMicrotask(() => {
            dispatchActionHintsChanged(document.activeElement);
        });
    }).observe(document.body, {
        subtree: true,
        childList: true,
        attributes: true,
        attributeFilter: [
            ACTIONS_ATTRIBUTE,
            SCOPE_ACTIONS_ATTRIBUTE,
            "data-focus-scope",
            "data-focusable",
            "data-focus-region",
        ],
    });

    window.__wolfUiActionHints = () => actionHintsFor(document.activeElement);

    dispatchActionHintsChanged(document.activeElement);

    function activateFocused() {
        const active = document.activeElement;
        if (!active || !isVisible(active)) return false;
        if (active.matches('button,a,input,select,textarea,[role="button"]')) {
            active.click();
            return true;
        }
        return false;
    }

    window.__wolfUiDispatchAction = (action) => {
        usingMouse = false;
        switch (action) {
            case "accept":
                if (!ensureFocusableActiveElement()) break;
                if (dispatchRustAction(action) || activateFocused()) {
                    uiSounds.play("select");
                }
                break;
            case "left":
            case "right":
            case "up":
            case "down":
                if (dispatchRustAction(action)) {
                    uiSounds.play("navigate");
                    break;
                }
                if (!document.activeElement?.matches?.(FOCUSABLE_SELECTOR) || !isVisible(document.activeElement)) {
                    if (ensureFocusableActiveElement()) uiSounds.play("navigate");
                    break;
                }
                if (moveFocus(action)) uiSounds.play("navigate");
                break;
            case "page-up":
                if (dispatchRustAction(action)) {
                    uiSounds.play("navigate");
                    break;
                }
                if (window.__wolfUiScrollPage?.(document.activeElement, -1)) uiSounds.play("navigate");
                break;
            case "page-down":
                if (dispatchRustAction(action)) {
                    uiSounds.play("navigate");
                    break;
                }
                if (window.__wolfUiScrollPage?.(document.activeElement, 1)) uiSounds.play("navigate");
                break;
            case "cancel":
                if (dispatchRustAction(action)) {
                    uiSounds.play("back");
                    break;
                }
                document.dispatchEvent(new CustomEvent("wolf-ui-cancel"));
                break;
            case "menu":
                if (dispatchRustAction(action)) {
                    uiSounds.play("navigate");
                    break;
                }
                document.dispatchEvent(new CustomEvent("wolf-ui-menu"));
                break;
        }
    };
})();
