import { useQuery } from "@tanstack/react-query";
import { api } from "./client";

export type HealthState =
  | { kind: "ok" }
  | { kind: "db-down" }
  | { kind: "unreachable" };

export async function fetchHealth(): Promise<HealthState> {
  try {
    const { data, error } = await api.GET("/api/health");
    if (data) return { kind: "ok" };
    // Proxies may answer with a non-JSON body, so don't trust the declared type.
    const body: unknown = error;
    const isDbDown =
      typeof body === "object" && body !== null && "db" in body && body.db === "down";
    return { kind: isDbDown ? "db-down" : "unreachable" };
  } catch {
    return { kind: "unreachable" };
  }
}

export function useHealth() {
  return useQuery({
    queryKey: ["health"],
    queryFn: fetchHealth,
    refetchInterval: 5_000,
    retry: false,
  });
}
