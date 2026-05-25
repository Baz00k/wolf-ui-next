(() => {
    if (window.__wolfUiFocusInstalled) return;
    window.__wolfUiFocusInstalled = true;

    const FOCUSABLE_SELECTOR = '[data-focusable="true"]';

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
        return true;
    }

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
                window.scrollBy({ top: -window.innerHeight * 0.8, behavior: "smooth" });
                break;
            case "page-down":
                window.scrollBy({ top: window.innerHeight * 0.8, behavior: "smooth" });
                break;
            case "cancel":
                document.dispatchEvent(new CustomEvent("wolf-ui-cancel"));
                break;
            case "menu":
                document.dispatchEvent(new CustomEvent("wolf-ui-menu"));
                break;
        }
    };
})();
