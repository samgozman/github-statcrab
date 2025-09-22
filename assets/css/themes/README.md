# How to add new themes?

If you want to contribute a new theme, please add a new CSS file in the `assets/css/themes` directory. The file name should be in kebab-case (e.g., `new-theme.css`). The macro will automatically generate the necessary Rust code for the new theme based on the file name.

The CSS classes defined in the theme file should follow the naming convention used in the existing themes.

> [!NOTE]  
> While you can use CSS for styling, keep in mind that you are working with SVG elements. This means that some CSS properties may not work as expected.

The **Transparent** column shows theme variants with `hide_background=true` and `hide_background_stroke=true` options enabled, removing the card background for integration into custom layouts.

## Stats Card

| Theme | Default | Transparent |
|-------|---------|-------------|
| `dark` | ![dark](examples/stats-card-dark.svg) | ![dark transparent](examples/stats-card-dark-transparent.svg) |
| `light` | ![light](examples/stats-card-light.svg) | ![light transparent](examples/stats-card-light-transparent.svg) |
| `monokai` | ![monokai](examples/stats-card-monokai.svg) | ![monokai transparent](examples/stats-card-monokai-transparent.svg) |
| `transparent_blue` | ![transparent_blue](examples/stats-card-transparent_blue.svg) | ![transparent_blue transparent](examples/stats-card-transparent_blue-transparent.svg) |

## Langs Card

| Theme | Default | Transparent |
|-------|---------|-------------|
| `dark` | ![dark](examples/langs-card-dark.svg) | ![dark transparent](examples/langs-card-dark-transparent.svg) |
| `light` | ![light](examples/langs-card-light.svg) | ![light transparent](examples/langs-card-light-transparent.svg) |
| `monokai` | ![monokai](examples/langs-card-monokai.svg) | ![monokai transparent](examples/langs-card-monokai-transparent.svg) |
| `transparent_blue` | ![transparent_blue](examples/langs-card-transparent_blue.svg) | ![transparent_blue transparent](examples/langs-card-transparent_blue-transparent.svg) |
