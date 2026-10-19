import { Info, PlugZap } from "lucide-react";
import { type NoticeView, WARNING_STYLE } from "@/design";

/** The notice kinds only Time pages use, added to the built-in ones through `Notice`'s `views` prop. */
export const TIME_NOTICE_VIEWS = {
  "not-connected": { icon: PlugZap, tone: "bg-module/15 text-module", surface: "", role: "status", spins: false },
  "settings-fallback": { icon: Info, ...WARNING_STYLE },
} as const satisfies Record<string, NoticeView>;
