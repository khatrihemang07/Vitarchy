import js from "@eslint/js";
import boundaries from "eslint-plugin-boundaries";
import reactHooks from "eslint-plugin-react-hooks";
import globals from "globals";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["dist", "src/api/schema.d.ts", "openapi.json"] },
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  {
    languageOptions: {
      globals: globals.browser,
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
    plugins: { "react-hooks": reactHooks, boundaries },
    rules: reactHooks.configs.recommended.rules,
  },
  {
    // Module isolation: src/modules/<a> may use its own files, the design
    // barrel, the api layer and the manifest type (app/module-manifest.ts).
    // It may not import another Module or any other part of app/.
    files: ["src/**/*.{ts,tsx}"],
    settings: {
      "import/resolver": {
        typescript: { project: "./tsconfig.json" },
        node: true,
      },
      // Folder mode is the default, so nested paths match src/modules/<id>/**.
      "boundaries/elements": [
        { type: "module", pattern: "src/modules/*", capture: ["id"] },
        { type: "app", pattern: "src/app" },
        { type: "api", pattern: "src/api" },
        { type: "design", pattern: "src/design" },
      ],
    },
    rules: {
      "boundaries/dependencies": [
        "error",
        {
          default: "allow",
          policies: [
            {
              // design/ is presentational: no data access, no app or Module knowledge.
              from: { element: { type: "design" } },
              disallow: {
                to: { element: { type: ["app", "api", "module"] } },
              },
            },
            {
              // Modules never see another Module...
              from: { element: { type: "module" } },
              disallow: {
                to: {
                  element: {
                    type: "module",
                    captured: { id: "!{{ from.element.captured.id }}" },
                  },
                },
              },
            },
            {
              // ...nor app internals (shell, dev, home, router, ...) beyond the manifest type.
              from: { element: { type: "module" } },
              disallow: {
                to: {
                  element: { type: "app" },
                  // The one public part of app/ is the manifest type.
                  file: { path: "!**/app/module-manifest.ts" },
                },
              },
            },
          ],
        },
      ],
    },
  },
  {
    // Outside the design system, import from the "@/design" barrel only.
    files: ["src/app/**/*.{ts,tsx}", "src/modules/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["@/design/*", "**/design/*"],
              message: 'Import from "@/design" (the public barrel), not deep paths.',
            },
          ],
        },
      ],
    },
  },
  {
    // Inside the design system, import siblings by relative path. Only the
    // shadcn / Magic UI copies keep the "@/design/..." aliases their generator
    // writes, so they stay diffable against upstream.
    files: ["src/design/**/*.{ts,tsx}"],
    ignores: ["src/design/ui/**", "src/design/effects/**"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["@/design", "@/design/*"],
              message: "Use a relative import for files inside the design system.",
            },
          ],
        },
      ],
    },
  },
  {
    // Code copied in from shadcn / Magic UI stays close to upstream so it can
    // be re-synced; skip the stylistic rules it was not written against.
    files: ["src/design/ui/**", "src/design/effects/**"],
    rules: {
      "@typescript-eslint/restrict-template-expressions": "off",
      "@typescript-eslint/no-confusing-void-expression": "off",
    },
  },
  {
    files: ["eslint.config.js"],
    ...tseslint.configs.disableTypeChecked,
  },
);
