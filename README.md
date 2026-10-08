# pvas-web-shared

Everything Private Asylum's web projects share: Dioxus components, the Yeti
build, writing and previewing a static site, and product documentation from
PVAS-DocGen manifests. Dioxus is used as a renderer only: pages become static
HTML at build time, with no WASM and no hydration.

## Crates

| Crate | Folder | What it is |
| --- | --- | --- |
| `pvas-web-shared` | `Crates/pvas-web-shared/` | The facade: re-exports the four below as `components`, `assets`, `site`, `docs` |
| `pvas-web-components` | `Crates/components/` | Document, nav, footer, intro, card: Yeti's markup in the house style, as Dioxus components |
| `pvas-web-assets` | `Crates/assets/` | The Yeti build and the house style with its typeface, embedded with `include_bytes!` |
| `pvas-web-site` | `Crates/site/` | `Site` + `write`, the `build`/`serve` command line, the Pagefind step, the local preview server |
| `pvas-web-docs` | `Crates/docs/` | Product documentation: the DocGen manifest model, markdown (comrak, syntect highlighting), the docs layout, reference and concept pages, the link check |
| `pvas-docs` | `Crates/docs-cli/` | Binary: builds or previews a product docs site from its `docs.toml` |

## Using it from a site

Pin a commit, never a branch, so a change here reaches a site only when that
site moves its pin:

```toml
[workspace.dependencies]
pvas-web-shared = { git = "https://github.com/Private-Asylum/pvas-web-shared", rev = "<commit>" }
```

A site's binary is then a few lines: render its pages, collect them with its
own static files into a `Site`, and hand that to `pvas_web_shared::site::run`,
which provides `build [--out <dir>]` and `serve [--out <dir>] [--port <port>]`.
`serve` previews the site under its base path (`Site::with_base`), as Pages
serves it. A site built `with_search` is indexed by Pagefind after writing,
through `npx` at the version pinned in `PAGEFIND_VERSION`.

## Product documentation sites

A product's docs repository holds only content: `docs.toml` (see the
`config` module of `pvas-web-docs` for every key), `content/*.md`, the
PVAS-DocGen `manifest.json` and an optional `styles/theme.css`. Run from that
repository:

```bash
cargo run --release --manifest-path <path-to>/pvas-web-shared/Cargo.toml -p pvas-docs -- [serve]
```

It renders the landing page, the hand-written pages, concept and reference
pages from the manifest, and a not-found page; checks every internal link and
anchor (a broken one fails the build); and indexes the output with Pagefind,
whose search trigger sits in every page's bar. Node must be installed.

## House style

`Crates/assets/house/house.css` is the Private Asylum look: a terminal,
lightly. One monospaced face, Geist Mono (self-hosted, OFL-1.1), a dark
screen with a faint phosphor glow and dot grid, hairline borders and
near-square corners. It is Yeti tokens and `yeti.theme` element rules, plus a
few unlayered rules for what Yeti has no token for, and a `.pa-cursor` class
for a blinking block cursor. It is not optional: `Site::with_house()` is the
only way a site gets Yeti, and it ships the house style beside it; pages link
`HOUSE_STYLESHEETS` (Yeti, then the house) before the site's own `theme.css`,
which sets only the hues: `--yeti-hue-primary` is the phosphor. A change to the
look is made here and reaches every site.

The shared components carry the house classes: `Intro` (a page's opening:
`~/eyebrow`, the headline ending in the cursor, a lede, the actions) and `Card`
(a whole-card link with an arrow, or a plain article with a badge), beside
`Nav` and `Footer`. `pa-prose` spaces flowing text and marks its headings
`##`/`###`.

The typeface is vendored at a pinned Fontsource version:

```bash
scripts/update-fonts.sh <version>
```

`vendor/fonts/geist-mono/VENDORED` records the version. A test fails if the
stylesheet and the embedded fonts disagree.

## Yeti

Yeti has no npm release yet and does not commit its `dist/`, so it is built at
a pinned commit and the output is committed under `Crates/assets/vendor/yeti/`:

```bash
scripts/update-yeti.sh <40-character commit sha>
```

`vendor/yeti/VENDORED` records the commit. A test fails if an update brings a
module or theme the embedded list does not cover. Yeti is FSL-1.1-MIT (any use
but a competing product; MIT after two years); its licence ships with it.

## Rules

- **Workspace layout:** crates under `Crates/`, no `src/`, each crate root
  beside its `Cargo.toml`, every module in `name/name.rs` wired with `#[path]`.
- **Full strict:** clippy `all`, `pedantic`, `nursery` and `cargo` denied,
  plus selected `restriction` lints and strict rustc and rustdoc lints.
  Exceptions use `#[expect(..., reason = "...")]`; `#[allow]` is itself denied.
  Proper nouns for `doc_markdown` live in `clippy.toml`.
- **CI** (`.github/workflows/check.yml`) runs fmt, clippy, doc and tests on
  every push and pull request. Sites pin commits, so `main` must stay green.
