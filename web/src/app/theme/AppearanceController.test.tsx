import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, render, waitFor } from "@testing-library/react";
import { AppearanceController } from "./AppearanceController";
import { APPEARANCE_CACHE_KEY } from "./apply";

const root = document.documentElement;
const core = {
  display_name: "",
  theme: "system",
  motion: "system",
  week_start: "monday",
  clock: "system",
  date_format: "system",
};

function serve(overrides: Partial<typeof core>) {
  vi.stubGlobal("fetch", () =>
    Promise.resolve(
      new Response(JSON.stringify({ core: { ...core, ...overrides }, modules: {} }), {
        headers: { "content-type": "application/json" },
      }),
    ),
  );
}

function mount() {
  return render(
    <QueryClientProvider client={new QueryClient()}>
      <AppearanceController>
        <p>app</p>
      </AppearanceController>
    </QueryClientProvider>,
  );
}

/** A matchMedia whose dark preference the test can flip. */
function controllableMedia(initialDark: boolean) {
  let dark = initialDark;
  const listeners = new Set<() => void>();
  window.matchMedia = ((query: string) => ({
    get matches() {
      return query.includes("dark") && dark;
    },
    media: query,
    addEventListener: (_: string, l: () => void) => listeners.add(l),
    removeEventListener: (_: string, l: () => void) => listeners.delete(l),
  })) as unknown as typeof window.matchMedia;
  return (next: boolean) => {
    dark = next;
    listeners.forEach((l) => {
      l();
    });
  };
}

const originalMatchMedia = window.matchMedia.bind(window);
afterEach(() => {
  window.matchMedia = originalMatchMedia;
  localStorage.clear();
  root.removeAttribute("data-theme");
  root.removeAttribute("data-motion");
  vi.unstubAllGlobals();
});

describe("AppearanceController", () => {
  it("applies the saved theme and motion", async () => {
    serve({ theme: "light", motion: "reduced" });
    mount();
    await waitFor(() => {
      expect(root).toHaveAttribute("data-motion", "reduced");
    });
    expect(root).toHaveAttribute("data-theme", "light");
  });

  it("follows the OS while the theme is System", async () => {
    const setDark = controllableMedia(false);
    serve({ theme: "system" });
    mount();
    await waitFor(() => {
      expect(root).toHaveAttribute("data-theme", "light");
    });
    act(() => {
      setDark(true);
    });
    expect(root).toHaveAttribute("data-theme", "dark");
  });

  it("uses the cached appearance before settings arrive, and when the server is down", () => {
    localStorage.setItem(APPEARANCE_CACHE_KEY, JSON.stringify({ theme: "dark", motion: "full" }));
    vi.stubGlobal("fetch", () => Promise.reject(new TypeError("offline")));
    mount();
    expect(root).toHaveAttribute("data-theme", "dark");
    expect(root).toHaveAttribute("data-motion", "full");
  });
});
