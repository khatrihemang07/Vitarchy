# Server-first, with Tauri later as a thin HTTP shell

Vitarchy is a Rust server plus a browser SPA that talks to it over HTTP. A Tauri desktop app may come later, but only as a thin shell that loads the same frontend and calls the same local server. We chose this so there is one source of truth for logic and data, and so the frontend stays a plain static SPA (no SSR) that any shell can wrap.

Vitarchy is local-only. The server binds to loopback (`127.0.0.1`) and refuses any other bind address. The frontend loads no external runtime resources: no CDN scripts or fonts, no analytics, no remote API calls. All assets are bundled and self-hosted. Installing packages for development is fine; nothing remote is used at runtime.
