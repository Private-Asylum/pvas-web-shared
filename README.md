# pvas-web-shared

Everything Private Asylum's web projects share: Dioxus components, the Yeti
build, writing and previewing a static site, and product documentation from
PVAS-DocGen manifests. Dioxus is used as a renderer only: pages become static
HTML at build time, with no WASM and no hydration.

## Crates

| Crate | Folder | What it is |
| --- | --- | --- |
| `pvas-web-shared` | `Crates/pvas-web-shared/` | The facade: re-exports the four below as `components`, `assets`, `site`, `docs` |
| `pvas-web-components` | `Crates/components/` | Document, nav, footer: Yeti's markup as Dioxus components |
| `pvas-web-assets` | `Crates/assets/` | The Yeti build, embedded with `include_bytes!` |
| `pvas-web-site` | `Crates/site/` | `Site` + `write`, the `build`/`serve` command line, the local preview server |
| `pvas-web-docs` | `Crates/docs/` | The DocGen manifest model (schema 1); the docs renderer grows here |

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
