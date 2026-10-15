import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { type AllSettings, type CoreSettings, useSaveSettings, useSettings } from "./settings";

const defaults: CoreSettings = {
  display_name: "",
  theme: "system",
  motion: "system",
  week_start: "monday",
  clock: "system",
  date_format: "system",
};
const initial: AllSettings = { core: defaults, modules: { time: { goal: 1 } } };

const json = (status: number, body: unknown) =>
  Promise.resolve(
    new Response(JSON.stringify(body), {
      status,
      headers: { "content-type": status >= 400 ? "application/problem+json" : "application/json" },
    }),
  );

/**
 * A fake server: GET answers with the current state; a PUT applies its body
 * (as the real server would) once `putResponse` resolves OK.
 */
function stubServer(putResponse: (putNumber: number) => Promise<Response>) {
  let puts = 0;
  let getGate: Promise<void> | undefined;
  let state: AllSettings = initial;
  const fetchMock = vi.fn(async (request: Request) => {
    if (request.method === "GET") {
      const snapshot = state; // a slow GET answers with the state as of when it was asked
      await getGate;
      return json(200, snapshot);
    }
    const body = (await request.clone().json()) as CoreSettings & Record<string, unknown>;
    const response = await putResponse(puts++);
    if (response.ok) {
      const id = /\/modules\/([^/]+)$/.exec(request.url)?.[1];
      state = id
        ? { ...state, modules: { ...state.modules, [id]: body } }
        : { ...state, core: body };
    }
    return response;
  });
  vi.stubGlobal("fetch", fetchMock);
  return Object.assign(fetchMock, {
    serverState: () => state,
    /** Makes later GETs wait; returns the function that lets them answer. */
    holdGets: () => {
      let open: () => void = () => undefined;
      getGate = new Promise<void>((resolve) => (open = resolve));
      return open;
    },
  });
}

/** Mounts the settings query and a save hook for `scope`, with the cache loaded. */
async function mountSave(scope: string) {
  const { wrapper } = setup();
  const settings = renderHook(() => useSettings(), { wrapper });
  await waitFor(() => {
    expect(settings.result.current.data).toBeDefined();
  });
  const save = renderHook(() => useSaveSettings(scope), { wrapper });
  return { settings, save };
}

const putsOf = (fetchMock: ReturnType<typeof stubServer>) =>
  fetchMock.mock.calls.map(([r]) => r).filter((r) => r.method === "PUT");

function setup() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return { client, wrapper };
}

afterEach(() => vi.unstubAllGlobals());

describe("useSettings", () => {
  it("loads all settings", async () => {
    stubServer(() => json(200, {}));
    const { wrapper } = setup();
    const { result } = renderHook(() => useSettings(), { wrapper });
    await waitFor(() => {
      expect(result.current.data).toEqual(initial);
    });
  });
});

describe("useSaveSettings", () => {
  it("updates the cache before the server answers, then keeps the saved value", async () => {
    let release: (r: Response) => void = () => undefined;
    const fetchMock = stubServer(() => new Promise<Response>((resolve) => (release = resolve)));
    const { settings, save } = await mountSave("core");

    act(() => {
      save.result.current.save("theme", "dark");
    });
    await waitFor(() => {
      expect(settings.result.current.data?.core.theme).toBe("dark");
    });
    expect(save.result.current.state).toEqual({ kind: "saving" });

    await act(async () => {
      release(await json(200, {}));
    });
    await waitFor(() => {
      expect(save.result.current.state).toEqual({ kind: "saved" });
    });
    expect(settings.result.current.data?.core.theme).toBe("dark");
    const [put] = putsOf(fetchMock);
    expect(put?.url).toContain("/api/settings/core");
    expect(await put?.clone().json()).toEqual({ ...defaults, theme: "dark" });
  });

  it("stores a Module's values under its id, merging only the changed key", async () => {
    const fetchMock = stubServer(() => json(200, {}));
    const { settings, save } = await mountSave("time");

    act(() => {
      save.result.current.save("auto", true);
    });
    await waitFor(() => {
      expect(save.result.current.state).toEqual({ kind: "saved" });
    });
    const [put] = putsOf(fetchMock);
    expect(put?.url).toContain("/api/settings/modules/time");
    expect(await put?.clone().json()).toEqual({ goal: 1, auto: true });
    expect(settings.result.current.data?.modules.time).toEqual({ goal: 1, auto: true });
  });

  it("persists two rapid changes made before the first one has been answered", async () => {
    const fetchMock = stubServer(() => json(200, {}));
    const { settings, save } = await mountSave("core");

    act(() => {
      save.result.current.save("theme", "dark");
      save.result.current.save("motion", "reduced");
    });
    await waitFor(() => {
      expect(save.result.current.state).toEqual({ kind: "saved" });
    });
    expect(fetchMock.serverState().core).toEqual({ ...defaults, theme: "dark", motion: "reduced" });
    expect(settings.result.current.data?.core).toEqual(fetchMock.serverState().core);
  });

  it("builds each change from the latest cache, not from when the hook rendered", async () => {
    const fetchMock = stubServer(() => json(200, {}));
    const { settings, save } = await mountSave("core");
    const staleSave = save.result.current.save;

    act(() => {
      staleSave("theme", "dark");
    });
    // Same function from the earlier render: it must still see the theme change.
    act(() => {
      staleSave("motion", "full");
    });
    await waitFor(() => {
      expect(save.result.current.state).toEqual({ kind: "saved" });
    });
    expect(settings.result.current.data?.core).toEqual({ ...defaults, theme: "dark", motion: "full" });
    expect(fetchMock.serverState().core.theme).toBe("dark");
  });

  it("when the first save fails and the second succeeds, ends on the server's state and shows the error", async () => {
    const fetchMock = stubServer((n) =>
      n === 0
        ? json(400, { title: "Bad Request", status: 400, detail: "name too long" })
        : json(200, {}),
    );
    const { settings, save } = await mountSave("core");

    act(() => {
      save.result.current.save("display_name", "x");
      save.result.current.save("theme", "dark");
    });
    await waitFor(() => {
      expect(save.result.current.state.kind).not.toBe("saving");
    });
    await waitFor(() => {
      expect(settings.result.current.data?.core).toEqual(fetchMock.serverState().core);
    });
    expect(save.result.current.state).toEqual({ kind: "error", message: "name too long" });
  });

  it("clears the error once a later save succeeds", async () => {
    stubServer((n) => (n === 0 ? json(400, { detail: "nope" }) : json(200, {})));
    const { save } = await mountSave("core");

    act(() => {
      save.result.current.save("theme", "dark");
    });
    await waitFor(() => {
      expect(save.result.current.state.kind).toBe("error");
    });
    act(() => {
      save.result.current.save("theme", "light");
    });
    await waitFor(() => {
      expect(save.result.current.state).toEqual({ kind: "saved" });
    });
  });

  it("does not let a slow, older read overwrite a change made while it was in flight", async () => {
    const fetchMock = stubServer(() => json(200, {}));
    const { settings, save } = await mountSave("core");

    const release = fetchMock.holdGets();
    void settings.result.current.refetch(); // asks the server as it is NOW: theme "system"
    act(() => {
      save.result.current.save("theme", "dark");
    });
    release(); // the older answer arrives after the change
    await waitFor(() => {
      expect(save.result.current.state).toEqual({ kind: "saved" });
    });
    await waitFor(() => {
      expect(settings.result.current.data?.core.theme).toBe("dark");
    });
    expect(fetchMock.serverState().core.theme).toBe("dark");
  });

  it("applies answers that arrive late without disturbing the cache", async () => {
    const releases: ((r: Response) => void)[] = [];
    const fetchMock = stubServer(() => new Promise<Response>((resolve) => releases.push(resolve)));
    const { settings, save } = await mountSave("core");

    act(() => {
      save.result.current.save("theme", "dark");
      save.result.current.save("motion", "reduced");
    });
    await waitFor(() => {
      expect(releases).toHaveLength(1);
    });
    await act(async () => {
      releases[0]?.(await json(200, {}));
    });
    await waitFor(() => {
      expect(releases).toHaveLength(2); // the second save starts only after the first is answered
    });
    await act(async () => {
      releases[1]?.(await json(200, {}));
    });
    await waitFor(() => {
      expect(save.result.current.state).toEqual({ kind: "saved" });
    });
    expect(settings.result.current.data?.core).toEqual({ ...defaults, theme: "dark", motion: "reduced" });
    expect(fetchMock.serverState().core).toEqual(settings.result.current.data?.core);
  });

  it("refetches once, when the last pending save settles", async () => {
    const fetchMock = stubServer(() => json(200, {}));
    const { save } = await mountSave("core");

    act(() => {
      save.result.current.save("theme", "dark");
      save.result.current.save("motion", "reduced");
    });
    await waitFor(() => {
      expect(save.result.current.state).toEqual({ kind: "saved" });
    });
    const gets = fetchMock.mock.calls.filter(([r]) => r.method === "GET");
    expect(gets).toHaveLength(2); // the initial load + one refetch
  });

  it("restores the server's state and shows the problem detail on a 400", async () => {
    stubServer(() => json(400, { title: "Bad Request", status: 400, detail: "name too long" }));
    const { settings, save } = await mountSave("core");

    act(() => {
      save.result.current.save("display_name", "x");
    });
    await waitFor(() => {
      expect(save.result.current.state).toEqual({ kind: "error", message: "name too long" });
    });
    await waitFor(() => {
      expect(settings.result.current.data?.core.display_name).toBe("");
    });
  });

  it("treats a 502 from a proxy as the server being unreachable", async () => {
    stubServer(() => Promise.resolve(new Response("", { status: 502 })));
    const { save } = await mountSave("core");

    act(() => {
      save.result.current.save("theme", "light");
    });
    await waitFor(() => {
      expect(save.result.current.state.kind).toBe("error");
    });
    const { state } = save.result.current;
    expect(state.kind === "error" && state.message).toMatch(/reach the server/i);
  });

  it("restores the server's state on a network error", async () => {
    stubServer(() => Promise.reject(new TypeError("Failed to fetch")));
    const { settings, save } = await mountSave("core");

    act(() => {
      save.result.current.save("theme", "light");
    });
    await waitFor(() => {
      expect(save.result.current.state.kind).toBe("error");
    });
    await waitFor(() => {
      expect(settings.result.current.data?.core.theme).toBe("system");
    });
  });
});
