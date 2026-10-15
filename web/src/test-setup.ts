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
