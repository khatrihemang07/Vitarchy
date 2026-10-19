import { QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { useState } from "react";
import { createQueryClient } from "@/api/queryClient";
import { TooltipProvider } from "@/design";
import { modules } from "./modules";
import { createAppRouter } from "./router";
import { AppearanceController } from "./theme/AppearanceController";

/** Root of the app: data, appearance, tooltips and the router. */
export function App() {
  const [queryClient] = useState(createQueryClient);
  const [router] = useState(() => createAppRouter(modules, { queryClient }));
  return (
    <QueryClientProvider client={queryClient}>
      <AppearanceController>
        <TooltipProvider>
          <RouterProvider router={router} />
        </TooltipProvider>
      </AppearanceController>
    </QueryClientProvider>
  );
}
