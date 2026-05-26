(() => {
    if (window.__wolfUiScrollAnimationInstalled) return;
    window.__wolfUiScrollAnimationInstalled = true;

    const activeScrolls = new WeakMap();

    function nearestHorizontalScroller(element) {
        let current = element?.parentElement;

        while (current && current !== document.body) {
            const style = window.getComputedStyle(current);
            const canScrollX =
                /(auto|scroll|overlay)/.test(style.overflowX) && current.scrollWidth > current.clientWidth;

            if (canScrollX) return current;
            current = current.parentElement;
        }

        return null;
    }

    function targetScrollLeft(container, element, inline = "center") {
        const containerRect = container.getBoundingClientRect();
        const elementRect = element.getBoundingClientRect();
        const current = container.scrollLeft;
        const elementLeft = elementRect.left - containerRect.left + current;
        const elementRight = elementLeft + elementRect.width;

        if (inline === "nearest") {
            const visibleLeft = current;
            const visibleRight = current + container.clientWidth;

            if (elementLeft >= visibleLeft && elementRight <= visibleRight) return current;

            const alignLeft = elementLeft;
            const alignRight = elementRight - container.clientWidth;
            return Math.abs(alignLeft - current) < Math.abs(alignRight - current) ? alignLeft : alignRight;
        }

        return elementLeft - (container.clientWidth - elementRect.width) / 2;
    }

    function clampScrollLeft(container, value) {
        return Math.max(0, Math.min(value, container.scrollWidth - container.clientWidth));
    }

    function easeOutCubic(progress) {
        return 1 - Math.pow(1 - progress, 3);
    }

    function scrollIntoHorizontalView(element, options = {}) {
        if (!element) return false;

        const container = nearestHorizontalScroller(element);
        if (!container) {
            element.scrollIntoView({ block: "nearest", inline: "nearest", behavior: "instant" });
            return true;
        }

        const target = clampScrollLeft(container, targetScrollLeft(container, element, options.inline ?? "center"));
        const existing = activeScrolls.get(container);
        if (existing?.frame) cancelAnimationFrame(existing.frame);

        const start = container.scrollLeft;
        const delta = target - start;
        const duration = options.duration ?? 220;
        const originalSnapType = existing?.originalSnapType ?? container.style.scrollSnapType;

        container.style.scrollSnapType = "none";

        function finishScroll() {
            container.scrollLeft = target;
            container.style.scrollSnapType = originalSnapType;
            activeScrolls.delete(container);
        }

        if (Math.abs(delta) < 1 || window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
            finishScroll();
            return true;
        }

        const state = { frame: null, startedAt: null, originalSnapType };

        function step(timestamp) {
            if (state.startedAt === null) state.startedAt = timestamp;

            const elapsed = timestamp - state.startedAt;
            const progress = Math.min(1, elapsed / duration);
            container.scrollLeft = start + delta * easeOutCubic(progress);

            if (progress < 1) {
                state.frame = requestAnimationFrame(step);
            } else {
                finishScroll();
            }
        }

        state.frame = requestAnimationFrame(step);
        activeScrolls.set(container, state);
        return true;
    }

    window.__wolfUiScrollIntoHorizontalView = scrollIntoHorizontalView;
    window.__wolfUiScrollSelectorIntoHorizontalView = (selector, options = {}) => {
        return scrollIntoHorizontalView(document.querySelector(selector), options);
    };
})();
