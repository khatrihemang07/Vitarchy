import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { useState } from "react";
import { TooltipProvider } from "@/design";
import { modules } from "./modules";
import { createAppRouter } from "./router";
import { AppearanceController } from "./theme/AppearanceController";

export function App() {
  const [queryClient] = useState(() => new QueryClient());
  const [router] = useState(() => createAppRouter(modules));
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
