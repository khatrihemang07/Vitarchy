import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { useState } from "react";
import { MotionProvider, TooltipProvider } from "@/design";
import { modules } from "./modules";
import { createAppRouter } from "./router";

export function App() {
  const [queryClient] = useState(() => new QueryClient());
  const [router] = useState(() => createAppRouter(modules));
  return (
    <QueryClientProvider client={queryClient}>
      <MotionProvider>
        <TooltipProvider>
          <RouterProvider router={router} />
        </TooltipProvider>
      </MotionProvider>
    </QueryClientProvider>
  );
}
