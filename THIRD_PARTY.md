# Third-Party Components and Artwork

## 7-Zip

The application bundles the official 7-Zip 26.03 Windows binaries, pinned in `tools/engine.lock.json`. Retain `runtime/7zip/License.txt` and the accompanying upstream files when distributing them. The license describes LGPL, BSD, and unRAR restrictions.

Upstream: https://github.com/ip7z/7zip

## Fluent UI System Icons

The SVG icons in `assets/fluent` come from `@fluentui/react-icons` 2.0.339, copyright Microsoft Corporation, under the MIT license. Regular exports were extracted from the original application prototype through React static rendering with their paths and view boxes preserved. The asset set also includes Filled variants imported by `tools/import-filled-icons.cjs`. Small UI glyphs receive their colors from the theme; file listings load Windows system icons. `HomeRegular.svg` uses the same package's `lib/atoms/svg/home.js` export.

The license is retained in `assets/fluent/LICENSE` and installed as `Fluent-LICENSE.txt`.

Upstream: https://github.com/microsoft/fluentui-system-icons

## Project Artwork

The brand artwork in `assets/brand/logo.svg` and the ten SVGs in `assets/toolbar` are original project drawings, not Microsoft or Motrix assets. The blue cursor depicts the current files page and is not a Cursor product logo. Appearance uses a palette, advanced settings a green bipyramid, and associations interlocking chain links. Toolbar SVGs are rasterized at the window scale, two samples per device pixel, then box-filtered back to the device-pixel grid and shown at 48 logical pixels. The history artwork also appears in empty states.

The brand SVG is the source for `logo.png` and the multi-resolution `app.ico`. The home view uses the PNG; Windows executable resources and the installer use the ICO. The titlebar uses text. Brand regeneration tools are listed in the README.

## Rust Dependencies

- GPUI: Apache-2.0, https://github.com/zed-industries/zed
- GPUI Kit: Apache-2.0, https://github.com/longbridge/gpui-kit
- ZIP comment support: zip 2.4.2, MIT, https://github.com/zip-rs/zip2

`Cargo.lock` records the complete dependency versions. This document records principal bundled components and artwork provenance; it is not a complete transitive dependency license inventory.
