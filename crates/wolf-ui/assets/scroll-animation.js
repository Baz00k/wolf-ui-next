(() => {
    if (window.__wolfUiScrollAnimationInstalled) return;
    window.__wolfUiScrollAnimationInstalled = true;

    const activeScrolls = new WeakMap();

    function cssPixels(value, referenceSize) {
        const pixels = Number.parseFloat(value);
        if (!Number.isFinite(pixels)) return 0;
        return value.trim().endsWith("%") ? (pixels / 100) * referenceSize : pixels;
    }

    function scrollPadding(container) {
        const style = window.getComputedStyle(container);
        return {
            top: cssPixels(style.scrollPaddingTop, container.clientHeight),
            right: cssPixels(style.scrollPaddingRight, container.clientWidth),
            bottom: cssPixels(style.scrollPaddingBottom, container.clientHeight),
            left: cssPixels(style.scrollPaddingLeft, container.clientWidth),
        };
    }

    function nearestScroller(element) {
        let current = element?.parentElement;

        while (current && current !== document.body) {
            const style = window.getComputedStyle(current);
            const canScrollX =
                /(auto|scroll|overlay)/.test(style.overflowX) && current.scrollWidth > current.clientWidth;
            const canScrollY =
                /(auto|scroll|overlay)/.test(style.overflowY) && current.scrollHeight > current.clientHeight;

            if (canScrollX || canScrollY) return current;
            current = current.parentElement;
        }

        return null;
    }

    function targetScrollLeft(container, element, padding, inline = "center") {
        const containerRect = container.getBoundingClientRect();
        const elementRect = element.getBoundingClientRect();
        const current = container.scrollLeft;
        const elementLeft = elementRect.left - containerRect.left + current;
        const elementRight = elementLeft + elementRect.width;

        if (inline === "nearest") {
            const visibleLeft = current + padding.left;
            const visibleRight = current + container.clientWidth - padding.right;

            if (elementLeft >= visibleLeft && elementRight <= visibleRight) return current;
            if (elementLeft <= visibleLeft && elementRight >= visibleRight) return current;

            const alignLeft = elementLeft - padding.left;
            const alignRight = elementRight - container.clientWidth + padding.right;
            return elementLeft < visibleLeft ? alignLeft : alignRight;
        }

        const visibleWidth = container.clientWidth - padding.left - padding.right;
        return elementLeft - padding.left - (visibleWidth - elementRect.width) / 2;
    }

    function clampScrollLeft(container, value) {
        return Math.max(0, Math.min(value, container.scrollWidth - container.clientWidth));
    }

    function targetScrollTop(container, element, padding, block = "nearest") {
        const containerRect = container.getBoundingClientRect();
        const elementRect = element.getBoundingClientRect();
        const current = container.scrollTop;
        const elementTop = elementRect.top - containerRect.top + current;
        const elementBottom = elementTop + elementRect.height;

        if (block === "center") {
            const visibleHeight = container.clientHeight - padding.top - padding.bottom;
            return elementTop - padding.top - (visibleHeight - elementRect.height) / 2;
        }

        const visibleTop = current + padding.top;
        const visibleBottom = current + container.clientHeight - padding.bottom;

        if (elementTop >= visibleTop && elementBottom <= visibleBottom) return current;
        if (elementTop <= visibleTop && elementBottom >= visibleBottom) return current;

        const alignTop = elementTop - padding.top;
        const alignBottom = elementBottom - container.clientHeight + padding.bottom;
        return elementTop < visibleTop ? alignTop : alignBottom;
    }

    function clampScrollTop(container, value) {
        return Math.max(0, Math.min(value, container.scrollHeight - container.clientHeight));
    }

    function easeOutCubic(progress) {
        return 1 - Math.pow(1 - progress, 3);
    }

    function scrollIntoView(element, options = {}) {
        if (!element) return false;

        const container = nearestScroller(element);
        if (!container) {
            element.scrollIntoView({ block: "nearest", inline: "nearest", behavior: "instant" });
            return true;
        }

        const padding = scrollPadding(container);
        const targetLeft = clampScrollLeft(
            container,
            targetScrollLeft(container, element, padding, options.inline ?? "nearest"),
        );
        const targetTop = clampScrollTop(
            container,
            targetScrollTop(container, element, padding, options.block ?? "nearest"),
        );
        const existing = activeScrolls.get(container);
        if (existing?.frame) cancelAnimationFrame(existing.frame);

        const startLeft = container.scrollLeft;
        const startTop = container.scrollTop;
        const deltaLeft = targetLeft - startLeft;
        const deltaTop = targetTop - startTop;
        const duration = options.duration ?? 220;
        const originalSnapType = existing?.originalSnapType ?? container.style.scrollSnapType;

        container.style.scrollSnapType = "none";

        function finishScroll() {
            container.scrollLeft = targetLeft;
            container.scrollTop = targetTop;
            container.style.scrollSnapType = originalSnapType;
            activeScrolls.delete(container);
        }

        if ((Math.abs(deltaLeft) < 1 && Math.abs(deltaTop) < 1) || window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
            finishScroll();
            return true;
        }

        const state = { frame: null, startedAt: null, originalSnapType };

        function step(timestamp) {
            if (state.startedAt === null) state.startedAt = timestamp;

            const elapsed = timestamp - state.startedAt;
            const progress = Math.min(1, elapsed / duration);
            const eased = easeOutCubic(progress);
            container.scrollLeft = startLeft + deltaLeft * eased;
            container.scrollTop = startTop + deltaTop * eased;

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

    function scrollPage(element, direction) {
        const container = nearestScroller(element) ?? document.scrollingElement;
        if (!container || direction === 0) return false;

        const maxScrollTop = container.scrollHeight - container.clientHeight;
        const targetTop = clampScrollTop(container, container.scrollTop + direction * container.clientHeight * 0.8);
        if (Math.abs(targetTop - container.scrollTop) < 1 || maxScrollTop <= 0) return false;

        container.scrollTo({
            top: targetTop,
            behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "instant" : "smooth",
        });
        return true;
    }

    window.__wolfUiScrollIntoView = scrollIntoView;
    window.__wolfUiScrollPage = scrollPage;
    window.__wolfUiScrollSelectorIntoView = (selector, options = {}) => {
        return scrollIntoView(document.querySelector(selector), options);
    };
})();
