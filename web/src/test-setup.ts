import "@testing-library/jest-dom/vitest";
window.scrollTo = () => undefined;

// jsdom has no IntersectionObserver (used by Motion's useInView).
class NoopIntersectionObserver {
  observe() {
    /* no-op */
  }
  unobserve() {
    /* no-op */
  }
  disconnect() {
    /* no-op */
  }
  takeRecords() {
    return [];
  }
}
window.IntersectionObserver = NoopIntersectionObserver as unknown as typeof IntersectionObserver;

// jsdom has no matchMedia; default to "no preference" (light, no reduced motion).
window.matchMedia = (query: string) =>
  ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    addListener: () => undefined,
    removeListener: () => undefined,
    dispatchEvent: () => false,
  }) as MediaQueryList;

// Radix Select / popovers call pointer-capture and scroll APIs jsdom lacks.
Element.prototype.hasPointerCapture = () => false;
Element.prototype.setPointerCapture = () => undefined;
Element.prototype.releasePointerCapture = () => undefined;
Element.prototype.scrollIntoView = () => undefined;

// cmdk observes its list height.
window.ResizeObserver = class {
  observe() {
    /* no-op */
  }
  unobserve() {
    /* no-op */
  }
  disconnect() {
    /* no-op */
  }
};
