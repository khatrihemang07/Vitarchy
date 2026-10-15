import { useNavigate } from "@tanstack/react-router";
import { Search, Sparkles } from "lucide-react";
import { useCallback, useState } from "react";
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  MotionButton,
} from "@/design";
import { HOME_NAV, type NavEntry } from "../nav";
import { useCommandHotkey } from "./useCommandHotkey";

/** Quick-add / jump bar. Navigation only until Modules contribute actions. */
export function CommandBar({ nav }: { nav: readonly NavEntry[] }) {
  const [open, setOpen] = useState(false);
  const navigate = useNavigate();
  const toggle = useCallback(() => {
    setOpen((o) => !o);
  }, []);
  useCommandHotkey(toggle);

  const go = (to: string) => {
    setOpen(false);
    void navigate({ to });
  };

  return (
    <>
      <MotionButton
        variant="secondary"
        onClick={toggle}
        className="h-11 w-full max-w-xl justify-start rounded-full px-4 font-normal text-muted-foreground shadow-key"
      >
        <Sparkles className="text-module" />
        <span className="flex-1 text-left">Quick add or jump to…</span>
        <kbd className="rounded-md border border-border px-1.5 font-mono text-xs">⌘K</kbd>
      </MotionButton>
      <CommandDialog open={open} onOpenChange={setOpen} title="Quick add" description="Jump to a screen.">
        <CommandInput placeholder="Where to?" />
        <CommandList>
          <CommandEmpty>
            <Search className="mx-auto mb-2 size-5" />
            Nothing matches yet.
          </CommandEmpty>
          <CommandGroup heading="Go to">
            <CommandItem
              onSelect={() => {
                go(HOME_NAV.to);
              }}
            >
              <HOME_NAV.icon /> {HOME_NAV.label}
            </CommandItem>
            {nav.map(({ id, label, to, icon: Icon }) => (
              <CommandItem
                key={id}
                onSelect={() => {
                  go(to);
                }}
              >
                <Icon /> {label}
              </CommandItem>
            ))}
          </CommandGroup>
        </CommandList>
      </CommandDialog>
    </>
  );
}
