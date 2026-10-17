import type { Facets } from "@/api/time";
import type { FacetKey } from "./reportActions";
import type { ReportFilters } from "./reportSearch";

/** One choice in a filter. */
export interface FacetOption {
  value: string;
  label: string;
  /** Extra word shown beside the label, e.g. "archived". */
  hint?: string;
}

/** Everything one filter picker needs: what it is, what it offers and what is chosen. */
export interface FacetDescriptor {
  key: FacetKey;
  label: string;
  options: FacetOption[];
  selected: string[];
}

/** The filters of the bar as data, so adding one is a new entry rather than new markup. */
export function facetDescriptors(facets: Facets, filters: ReportFilters): FacetDescriptor[] {
  return [
    {
      key: "project",
      label: "Project",
      options: facets.projects.map((p) => ({ value: String(p.id), label: p.name, ...(!p.active && { hint: "archived" }) })),
      selected: filters.project.map(String),
    },
    {
      key: "tag",
      label: "Tag",
      options: facets.tags.map((t) => ({ value: t.name, label: t.name })),
      selected: filters.tag,
    },
    {
      key: "client",
      label: "Client",
      options: facets.clients.map((c) => ({ value: String(c.id), label: c.name, ...(c.archived && { hint: "archived" }) })),
      selected: filters.client.map(String),
    },
  ];
}
