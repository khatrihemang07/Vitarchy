import { Check, ChevronDown } from "lucide-react";
import { useState } from "react";
import {
  Badge,
  Button,
  cn,
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/design";
import type { FacetDescriptor } from "./facetOptions";

interface FacetFilterProps {
  facet: FacetDescriptor;
  onChange: (values: string[]) => void;
}

/** A multi-select: any of the chosen values matches. The trigger shows how many are chosen. */
export function FacetFilter({ facet, onChange }: FacetFilterProps) {
  const [open, setOpen] = useState(false);
  const count = facet.selected.length;
  const toggle = (value: string) => {
    onChange(facet.selected.includes(value) ? facet.selected.filter((v) => v !== value) : [...facet.selected, value]);
  };
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="outline" size="sm" aria-label={`${facet.label} filter`} className={cn(count > 0 && "border-module/60")}>
          {facet.label}
          {count > 0 ? (
            <Badge className="px-1.5 tabular-nums">{count}</Badge>
          ) : null}
          <ChevronDown className="opacity-50" aria-hidden="true" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-64">
        <Command>
          <CommandInput placeholder={`Search ${facet.label.toLowerCase()}s…`} />
          <CommandList>
            <CommandEmpty>Nothing found.</CommandEmpty>
            <CommandGroup>
              {facet.options.map((option) => {
                const selected = facet.selected.includes(option.value);
                return (
                  <CommandItem
                    key={option.value}
                    value={`${option.label} ${option.value}`}
                    aria-selected={selected}
                    onSelect={() => {
                      toggle(option.value);
                    }}
                  >
                    <Check className={cn("size-4", selected ? "opacity-100" : "opacity-0")} aria-hidden="true" />
                    <span className="min-w-0 flex-1 truncate">{option.label}</span>
                    {option.hint ? <span className="text-xs text-muted-foreground">{option.hint}</span> : null}
                  </CommandItem>
                );
              })}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
