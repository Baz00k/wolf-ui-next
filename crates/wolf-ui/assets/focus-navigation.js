(() => {
    if (window.__wolfUiFocusInstalled) return;
    window.__wolfUiFocusInstalled = true;

    const FOCUSABLE_SELECTOR = '[data-focusable="true"]';
    const ACTIONS_ATTRIBUTE = "data-actions";
    const SCOPE_ACTIONS_ATTRIBUTE = "data-scope-actions";
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

    function isVisible(element) {
        const style = window.getComputedStyle(element);
        if (style.display === "none" || style.visibility === "hidden") return false;
        if (element.matches(':disabled,[aria-disabled="true"],[inert]')) return false;
        const rect = element.getBoundingClientRect();
        return rect.width > 0 && rect.height > 0;
    }

    function scopeFor(element) {
        return (
            element?.closest?.("[data-focus-scope]") ?? document.querySelector("[data-focus-scope]") ?? document.body
        );
    }

    function candidatesIn(scope) {
        return Array.from(scope.querySelectorAll(FOCUSABLE_SELECTOR)).filter(isVisible);
    }

    function center(rect) {
        return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
    }

    function directionScore(action, current, candidate) {
        const currentCenter = center(current);
        const candidateCenter = center(candidate);
        const dx = candidateCenter.x - currentCenter.x;
        const dy = candidateCenter.y - currentCenter.y;
        const minHorizontalStep = Math.min(current.width, candidate.width) * 0.25;
        const minVerticalStep = Math.min(current.height, candidate.height) * 0.25;

        switch (action) {
            case "left":
                if (dx >= -minHorizontalStep) return null;
                return Math.abs(dx) + Math.abs(dy) * 2;
            case "right":
                if (dx <= minHorizontalStep) return null;
                return Math.abs(dx) + Math.abs(dy) * 2;
            case "up":
                if (dy >= -minVerticalStep) return null;
                return Math.abs(dy) + Math.abs(dx) * 2;
            case "down":
                if (dy <= minVerticalStep) return null;
                return Math.abs(dy) + Math.abs(dx) * 2;
            default:
                return null;
        }
    }

    function focusFirst(scope) {
        const first = candidatesIn(scope)[0];
        if (!first) return false;
        first.focus({ preventScroll: true });
        first.scrollIntoView({ block: "nearest", inline: "nearest", behavior: "smooth" });
        dispatchActionHintsChanged(first);
        return true;
    }

    function ensureFocusableActiveElement() {
        if (document.activeElement?.matches?.(FOCUSABLE_SELECTOR) && isVisible(document.activeElement)) {
            return true;
        }

        return focusFirst(scopeFor(document.activeElement));
    }

    window.__wolfUiEnsureFocusableActiveElement = ensureFocusableActiveElement;

    function moveFocus(action) {
        const active = document.activeElement?.matches?.(FOCUSABLE_SELECTOR) ? document.activeElement : null;
        const scope = scopeFor(active);

        if (!active) return focusFirst(scope);

        const currentRect = active.getBoundingClientRect();
        let best = null;
        let bestScore = Infinity;

        for (const candidate of candidatesIn(scope)) {
            if (candidate === active) continue;
            const score = directionScore(action, currentRect, candidate.getBoundingClientRect());
            if (score !== null && score < bestScore) {
                best = candidate;
                bestScore = score;
            }
        }

        if (!best) return false;
        best.focus({ preventScroll: true });
        best.scrollIntoView({ block: "nearest", inline: "nearest", behavior: "smooth" });
        dispatchActionHintsChanged(best);
        return true;
    }

    function actionHintsFor(element) {
        const scope = scopeFor(element);
        const hints = new Map();

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
        document.dispatchEvent(
            new CustomEvent("wolf-ui-action-hints-changed", {
                detail: actionHintsFor(element?.matches?.(FOCUSABLE_SELECTOR) ? element : null),
            }),
        );
    }

    document.addEventListener("focusin", (event) => {
        dispatchActionHintsChanged(event.target);
    });

    document.addEventListener("focusout", () => {
        queueMicrotask(() => dispatchActionHintsChanged(document.activeElement));
    });

    new MutationObserver(() => {
        queueMicrotask(() => {
            ensureFocusableActiveElement();
            dispatchActionHintsChanged(document.activeElement);
        });
    }).observe(document.body, {
        subtree: true,
        childList: true,
        attributes: true,
        attributeFilter: [ACTIONS_ATTRIBUTE, SCOPE_ACTIONS_ATTRIBUTE, "data-focus-scope", "data-focusable"],
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
        switch (action) {
            case "accept":
                ensureFocusableActiveElement();
                if (dispatchRustAction(action)) break;
                activateFocused();
                break;
            case "left":
            case "right":
            case "up":
            case "down":
                if (!document.activeElement?.matches?.(FOCUSABLE_SELECTOR) || !isVisible(document.activeElement)) {
                    ensureFocusableActiveElement();
                    break;
                }
                moveFocus(action);
                break;
            case "page-up":
                if (dispatchRustAction(action)) break;
                window.scrollBy({ top: -window.innerHeight * 0.8, behavior: "smooth" });
                break;
            case "page-down":
                if (dispatchRustAction(action)) break;
                window.scrollBy({ top: window.innerHeight * 0.8, behavior: "smooth" });
                break;
            case "cancel":
                if (dispatchRustAction(action)) break;
                document.dispatchEvent(new CustomEvent("wolf-ui-cancel"));
                break;
            case "menu":
                if (dispatchRustAction(action)) break;
                document.dispatchEvent(new CustomEvent("wolf-ui-menu"));
                break;
        }
    };
})();
