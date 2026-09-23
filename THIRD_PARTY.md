# Third-Party Components and Artwork

## 7-Zip

The application bundles the official 7-Zip 26.03 Windows binaries, pinned in `tools/engine.lock.json`. Retain `runtime/7zip/License.txt` and the accompanying upstream files when distributing them. The license describes LGPL, BSD, and unRAR restrictions.

Upstream: https://github.com/ip7z/7zip

## Microsoft Fluent UI icons

The SVG icons in `assets/fluent` come from `@fluentui/react-icons` 2.0.339, copyright Microsoft Corporation, under the MIT license. Regular exports were extracted from the original application prototype through React static rendering with their paths and view boxes preserved. The asset set also includes Filled variants imported by `tools/import-filled-icons.cjs`. Small UI glyphs receive their colors from the theme; file listings load Windows system icons. `HomeRegular.svg` uses the same package's `lib/atoms/svg/home.js` export.

The license is retained in `assets/fluent/LICENSE` and installed as `Fluent-LICENSE.txt`.

Upstream: https://github.com/microsoft/fluentui-system-icons

`assets/fluent/ArrowClockwiseRegular.svg` was imported on 2026-09-23 from the same MIT-licensed upstream repository, `assets/Arrow Clockwise/SVG/ic_fluent_arrow_clockwise_20_regular.svg`, to restore the path-bar refresh glyph.

## Project Artwork

Cardo reuses `assets/toolbar/general.svg`, the purple Application cube, unchanged as `cardo/assets/logo.svg` under the project's MIT license. Its independent repository records the source in `cardo/THIRD_PARTY.md`.

The brand artwork in `assets/brand/logo.svg` and the SVGs in `assets/toolbar` are original project drawings, not Microsoft or Motrix assets. The browser page icon reuses `assets/brand/logo.svg`. The established folder, extraction arrow, lightning, shield, clock, purple cube, palette, green bipyramid, chain links and information badge remain the active icon family. Help-book and update-arrow artwork is retained but unused. The retained unused cursor is not a Cursor product logo. Toolbar SVGs are rasterized at the window scale, two samples per device pixel, then box-filtered back to the device-pixel grid and shown at 48 logical pixels. History artwork also appears in empty states.

The brand SVG is the source for `logo.png` and the multi-resolution `app.ico`. Home no longer displays a large logo; the browser toolbar retains the brand icon. Windows executable resources and the installer use the ICO. The titlebar uses text. `tools/render-brand.cjs` and `tools/generate-icon.py` regenerate these assets.

## Microsoft Visual C++ Runtime

`vcruntime140.dll` is copied from the x64 redistributable directory of the installed Visual Studio C++ toolset. The runnable folder, portable ZIP and installer include it for application-local loading. It remains Microsoft proprietary software distributed under the Visual Studio redistribution terms; it is not covered by the project's or Rust dependencies' licenses. Windows system DLLs are supplied by the supported operating system and are not copied into the packages.

Redistribution guidance: https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files

## Rust Dependencies

- GPUI: Apache-2.0, https://github.com/zed-industries/zed
- GPUI Kit: Apache-2.0, https://github.com/longbridge/gpui-kit
- GPUI Component 0.6.1: Apache-2.0, https://github.com/longbridge/gpui-component. Resolved from crates.io through GPUI Kit; provides the themed PopupMenu component, cascading and keyboard interactions.
- ZIP comment support: zip 2.4.2, MIT, https://github.com/zip-rs/zip2
- Localization engine: fluent-bundle and fluent-syntax, Apache-2.0 OR MIT, https://github.com/projectfluent/fluent-rs
- Diagnostics: tracing, tracing-subscriber and tracing-appender, MIT, https://github.com/tokio-rs/tracing
- Windows registry access: winreg 0.55, MIT, https://github.com/gentoo90/winreg-rs

`Cargo.lock` records the complete dependency versions. This document records principal bundled components and artwork provenance; it is not a complete transitive dependency license inventory.

## Project Artwork

`assets/toolbar/warning.svg` is an original derivative of the project's `about.svg` badge, retaining its blue dimensional face and replacing the information letter with an exclamation mark. It introduces no third-party assets.

`assets/toolbar/delete.svg` is original project artwork: a red dimensional wastebasket with separate front, side and lid faces, matching the established object-icon family. No third-party image source is used.

`assets/toolbar/error.svg` is an original red color variant of the project's exclamation badge, sharing the same dimensional geometry. Error, conflict, deletion and extraction headers use the common panel_artwork_notice component.
