import { LogoMark } from "@/design";

/** Logo and name at the top of the sidebar. */
export function Brand() {
  return (
    <div className="flex shrink-0 items-center gap-2.5 px-1">
      <LogoMark />
      <span className="font-display text-lg font-bold tracking-tight">Vitarchy</span>
    </div>
  );
}
