# Documentation site

The English and 简体中文 documentation is built with Astro and Starlight. Use **Bun 1.4.4** (pinned in `.bun-version`, `package.json`, and CI).

Run from `docs/`:

```sh
bun install --frozen-lockfile
bun run build:wasm
bun run dev
```

The site uses the `/gpui-rsx/` base path. Open the URL reported by Astro with that path, for example `http://localhost:4321/gpui-rsx/`.

The playground build needs Rust 1.98.1, the `wasm32-unknown-unknown` target, and `wasm-pack`. Generated files in `public/playground/pkg/` are ignored by Git. Rebuild them after changing the Rust parser, tools, or WASM crate.

Before submitting changes:

```sh
bun run check
bun run build
bun ../scripts/test-playground.mjs
```

`build` compiles the actual WASM parser, generates both locales and the search index, then checks internal page links, anchors, and asset paths. It uses `astro build --force` so cached Markdown cannot retain obsolete code-theme asset paths after configuration changes. `bun run check:links` can repeat the link check against an existing `dist/` directory. `bun run preview` serves the production output locally.

Content lives in `src/content/docs/`; Chinese translations mirror the same paths under `zh-cn/`. Keep examples, links, and version guidance aligned in both languages. Shared colors and reading styles live in `src/styles/custom.css`; the landing page and playground use the same visual tokens. They use system fonts and do not require external font requests.
