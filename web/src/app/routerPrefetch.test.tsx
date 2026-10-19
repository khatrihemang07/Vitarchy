import { QueryClient, QueryClientProvider, queryOptions } from "@tanstack/react-query";
import { createMemoryHistory, createRoute, RouterProvider } from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import type { ModuleManifest } from "./module-manifest";
import { warm, warmCache } from "./prefetch";
import { createAppRouter } from "./router";

const pageData = queryOptions({ queryKey: ["demo", "page"], queryFn: () => Promise.resolve("warmed"), retry: false });

const demo: ModuleManifest = {
  id: "demo",
  label: "Demo",
  accent: "#0af",
  icon: () => null,
  routes: (root) => [
    createRoute({ getParentRoute: () => root, path: "/demo", component: () => <p>demo page</p>, loader: warmCache([warm(pageData)]) }),
  ],
  navItem: { to: "/demo" },
};

function renderAt(path: string, queryClient: QueryClient) {
  const router = createAppRouter([demo], { includeDesignShowcase: false, queryClient });
  router.update({ history: createMemoryHistory({ initialEntries: [path] }), context: { queryClient } });
  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

describe("route loaders", () => {
  it("warm the app's query cache before the page shows", async () => {
    const queryClient = new QueryClient();
    renderAt("/demo", queryClient);
    expect(await screen.findByText("demo page")).toBeInTheDocument();
    expect(queryClient.getQueryData(pageData.queryKey)).toBe("warmed");
  });
});
