# How to add new themes?

If you want to contribute a new theme, please add a new CSS file in the `assets/css/themes` directory. The file name should be in kebab-case (e.g., `new-theme.css`). The macro will automatically generate the necessary Rust code for the new theme based on the file name.

The CSS classes defined in the theme file should follow the naming convention used in the existing themes.

> [!NOTE]  
> While you can use CSS for styling, keep in mind that you are working with SVG elements. This means that some CSS properties may not work as expected.

## Stats Card

| Theme | Example |
|-------|----------|
| `dark` | ![dark](examples/stats-card-dark.svg) |
| `light` | ![light](examples/stats-card-light.svg) |
| `monokai` | ![monokai](examples/stats-card-monokai.svg) |
| `transparent_blue` | ![transparent_blue](examples/stats-card-transparent_blue.svg) |

## Langs Card

| Theme | Example |
|-------|----------|
| `dark` | ![dark](examples/langs-card-dark.svg) |
| `light` | ![light](examples/langs-card-light.svg) |
| `monokai` | ![monokai](examples/langs-card-monokai.svg) |
| `transparent_blue` | ![transparent_blue](examples/langs-card-transparent_blue.svg) |
