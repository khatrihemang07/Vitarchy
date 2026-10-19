import { Check, ChevronDown } from "lucide-react";
import { useState } from "react";
import { cn } from "../lib/utils";
import { Badge } from "../ui/badge";
import { Button } from "../ui/button";
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from "../ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";

/** One choice in a multi-select. */
export interface FacetOption {
  value: string;
  label: string;
  /** Extra word shown beside the label, e.g. "archived". */
  hint?: string;
}

interface FacetFilterProps {
  /** What is being chosen, singular: "Project", "Account". Names the button and the search box. */
  label: string;
  options: readonly FacetOption[];
  selected: readonly string[];
  onChange: (values: string[]) => void;
}

/** A multi-select: any of the chosen values matches. The trigger shows how many are chosen. */
export function FacetFilter({ label, options, selected, onChange }: FacetFilterProps) {
  const [open, setOpen] = useState(false);
  const count = selected.length;
  const toggle = (value: string) => {
    onChange(selected.includes(value) ? selected.filter((v) => v !== value) : [...selected, value]);
  };
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="outline" size="sm" aria-label={`${label} filter`} className={cn(count > 0 && "border-module/60")}>
          {label}
          {count > 0 ? (
            <Badge className="px-1.5 tabular-nums">{count}</Badge>
          ) : null}
          <ChevronDown className="opacity-50" aria-hidden="true" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-64">
        <Command>
          <CommandInput placeholder={`Search ${label.toLowerCase()}s…`} />
          <CommandList>
            <CommandEmpty>Nothing found.</CommandEmpty>
            <CommandGroup>
              {options.map((option) => {
                const chosen = selected.includes(option.value);
                return (
                  <CommandItem
                    key={option.value}
                    value={`${option.label} ${option.value}`}
                    aria-selected={chosen}
                    onSelect={() => {
                      toggle(option.value);
                    }}
                  >
                    <Check className={cn("size-4", chosen ? "opacity-100" : "opacity-0")} aria-hidden="true" />
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
