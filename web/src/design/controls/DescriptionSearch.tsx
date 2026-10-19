import { Search } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Input } from "../ui/input";

/** Waits for a pause in typing so each keystroke does not become a request. */
const DEBOUNCE_MS = 300;

interface DescriptionSearchProps {
  /** The committed search (from the URL); a change that did not come from typing here replaces the text. */
  value: string;
  onSearch: (text: string) => void;
}

/** Searches descriptions, ignoring case; reports the text after a short pause. */
export function DescriptionSearch({ value, onSearch }: DescriptionSearchProps) {
  const [text, setText] = useState(value);
  // The last value this box reported or saw; a different `value` came from elsewhere (Clear filters, Back).
  const [seen, setSeen] = useState(value);
  if (value !== seen) {
    setSeen(value);
    setText(value);
  }
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  // The latest callback, so a pause that ends after other controls changed the URL does not undo them.
  const latest = useRef(onSearch);
  useEffect(() => {
    latest.current = onSearch;
  });
  useEffect(() => () => {
    clearTimeout(timer.current);
  }, []);
  return (
    <div className="relative w-full sm:w-64">
      <Search className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
      <Input
        type="search"
        aria-label="Search descriptions"
        placeholder="Search descriptions"
        className="h-8 pl-8 text-sm"
        value={text}
        onChange={(event) => {
          const next = event.target.value;
          setText(next);
          clearTimeout(timer.current);
          timer.current = setTimeout(() => {
            setSeen(next);
            latest.current(next);
          }, DEBOUNCE_MS);
        }}
      />
    </div>
  );
}
