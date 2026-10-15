# shadcn as an owned base, with five design layers on top

The frontend uses shadcn/ui on Tailwind v4 as an owned-copy base: components are generated into `web/src/design/ui/` and we edit nothing there that a token can express. Five layers give it a distinct look: CSS-variable theme tokens (`@theme`, per-module accent), Motion springs, copied-in effects from shadcn-compatible registries (Magic UI), self-hosted fonts (`@fontsource-variable`) and grain/glow texture. Everything is dark-only and imported through one barrel, `@/design`.

We chose this over Tailwind with React Aria and over CSS Modules because shadcn gives the fastest start on accessible primitives while theming and layers keep it from looking stock. The trade-off is that generated code lives in our repo and must be re-synced by hand, and a few upstream files carry lint exceptions.

Consistent with ADR 0001, nothing is fetched at runtime: fonts, texture and icons (lucide-react) are bundled.
