import type { Facets } from "@/api/time";
import { Button } from "@/design";
import { DescriptionSearch } from "./DescriptionSearch";
import { facetDescriptors } from "./facetOptions";
import { FacetFilter } from "./FacetFilter";
import type { FacetKey } from "./reportActions";
import type { ReportSearch } from "./reportSearch";

interface FilterBarProps {
  facets: Facets | undefined;
  search: ReportSearch;
  active: boolean;
  onFacet: (key: FacetKey, values: string[]) => void;
  onSearch: (text: string) => void;
  onClear: () => void;
}

/** Project, tag and client pickers plus a description search; they combine with AND, values inside one with OR. */
export function FilterBar({ facets, search, active, onFacet, onSearch, onClear }: FilterBarProps) {
  const descriptors = facets ? facetDescriptors(facets, search) : [];
  return (
    <div className="flex flex-wrap items-center gap-2" role="group" aria-label="Filters">
      {descriptors.map((facet) => (
        <FacetFilter
          key={facet.key}
          facet={facet}
          onChange={(values) => {
            onFacet(facet.key, values);
          }}
        />
      ))}
      <DescriptionSearch value={search.q} onSearch={onSearch} />
      {active ? (
        <Button variant="ghost" size="sm" onClick={onClear}>
          Clear filters
        </Button>
      ) : null}
    </div>
  );
}
