# card_theme_macros

Procedural macros that generate theme-related types for github-statcrab from CSS files in `assets/css/themes`.

## What it does

- `build_card_themes!()` generates:
  - `pub enum CardTheme { ... }` — one variant per `*.css` in `assets/css/themes` (filename → PascalCase).
  - `impl CardTheme { pub fn load_css(&self) -> &'static str }` — embeds each CSS using `include_str!`.
  - Variant Rustdocs are derived from the filename (e.g., `transparent-blue.css` → "Transparent Blue").

- `build_theme_query!()` generates:
  - `pub enum ThemeQuery` with `#[derive(serde::Deserialize)]` — one variant per theme.
  - `#[serde(rename = "...")]` uses the snake_case of the filename (e.g., `transparent_blue`).
  - `impl From<ThemeQuery> for CardTheme` for 1:1 mapping.

## Usage

In your crate where you need the types, invoke the macros at module scope:

```rust
use card_theme_macros::{build_card_themes, build_theme_query};

build_card_themes!();
build_theme_query!();

// Now you have CardTheme and ThemeQuery available.
```

Example integration points:

- Rendering cards: store a `CardTheme` on your settings and call `theme.load_css()` to get the CSS string.
- HTTP query: deserialize `ThemeQuery` from a `theme` query param and convert into `CardTheme` via `into()`.

## Adding a new theme

Just drop a new `*.css` file into `assets/css/themes`. The macros will generate new enum variants automatically.

Tip: ensure your main crate has a build script that watches the themes directory so changes trigger rebuilds:

```rust
// build.rs (in the main crate)
println!("cargo:rerun-if-changed=assets/css/themes");
```

## Notes

- Filenames are the single source of truth for display docs and query names.
- CSS is embedded at compile time; no runtime file I/O is performed for themes.
