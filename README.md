# 7zplus

[简体中文](README.zh.md)

7zplus is a native Windows archive manager written in Rust, using GPUI Kit 0.6.1 and the bundled 7-Zip 26.03 engine. Its interface combines a quiet background, an inset white content panel in the light theme, and large colored toolbar artwork.

## Features

- Browse archives and local folders, search entries, sort lists, select multiple items, and revisit recent locations.
- Create 7z, ZIP, TAR, gzip, bzip2, xz, and WIM archives with format-dependent compression, password, and volume options.
- Extract all or selected entries, choose overwrite behavior, run integrity checks, and cancel background operations.
- Add, rename, delete, copy out, and move out archive entries where the archive format supports editing. Read and write ZIP comments.
- Open extracted entries with their Windows application, calculate checksums, and generate or verify checksum files.
- Integrate with Explorer through a separate shell extension, optional per-user file associations, and single-instance command forwarding.
- Switch between Simplified Chinese, Traditional Chinese, and English, and between light and dark themes.

Archive operations run through the bundled engine. Opening an entry in another application does not write its later changes back into the archive.

## Build and Run

Requirements:

- Windows x64, Rust 1.95.0 with the MSVC target specified in `rust-toolchain.toml`.
- Visual Studio C++ Build Tools and the Windows SDK.
- PowerShell and network access for the initial dependency downloads.

```powershell
./tools/build.ps1
./bin/7zplus.exe
```

The script builds the application and shell extension, prepares the pinned 7-Zip and NSIS downloads with SHA-256 verification, and produces:

```text
bin/
  7zplus.exe
  7-zip-plus.dll
  7zplus-amd64-installer.exe
  SHA256SUMS.txt
  runtime/7zip/
```

Keep `runtime/7zip/` beside the executable. Use the installer for Explorer integration. Registration uses application-owned per-user entries and does not replace Windows' default archive association.

To enable GitHub release checks in a build, pass `-ReleaseRepository owner/repo` to the build script. User settings are stored in `%LOCALAPPDATA%\7zplus-rust`.

## Command Line

```text
7zplus.exe [--lang zh-CN|zh-TW|en-US] [path ...]
7zplus.exe --register [dll-path]
7zplus.exe --unregister
7zplus.exe --prepare-install <installation-directory>
```

The last three commands support installation and maintenance. Explorer action identifiers and their complete selection payload are defined in `crates/shell-api`.

Common shortcuts: `Ctrl+O` opens an archive, `Ctrl+L` focuses the address, `Ctrl+F` searches, `Ctrl+A` selects all entries, `Alt+Left` goes back, `Alt+Up` goes to the parent, `Alt+E` opens extraction options, `Alt+T` checks integrity, and `F1` opens the bundled help. Menus display additional shortcuts.

## Architecture

| Directory | Responsibility |
| --- | --- |
| `src/archive` | Archive models, the 7-Zip process adapter, editing, comments, checksums, and progress parsing. No GPUI or registry dependencies. |
| `src/application` | Background requests and outcomes, filesystem loading, extraction workflows, and release checks. |
| `src/platform` | Windows file opening and icons, single-instance IPC, registry integration, mail, and installation maintenance. |
| `src/settings`, `src/i18n.rs` | Persisted preferences, recent locations, and Fluent localization. |
| `src/ui/mod.rs` | Workspace state, subscriptions, initialization, and input/focus synchronization. |
| `src/ui/actions` | Task lifecycle and workspace coordination for archive operations and preferences. |
| `src/ui/views` | Window chrome, menu bar, file lists, and settings-page composition. |
| `src/ui/dialogs` | Dialog composition, archive creation form, and error presentation. |
| `src/ui/components` | Shared buttons, inputs, tooltips, navigation, lists, settings rows, icons, and menu styling. |
| `src/ui/theme`, `src/ui/theme.rs` | Shared dimensions, semantic colors, and GPUI theme integration. |
| `crates/shell-api` | Shared action enums, format routing, and serialized selection requests. |
| `crates/shell` | Explorer COM extension; independent of GPUI and the archive engine. |
| `locales`, `assets` | Product text and embedded visual resources. |
| `tools`, `packaging` | Reproducible engine preparation, build scripts, and the NSIS installer. |

UI commands dispatch background work and consume outcomes. Views compose components; they do not perform filesystem, process, network, or registry work. Context menus retain the clicked target and selection snapshot. Product text belongs in all three Fluent locale files, using semantic keys rather than translated labels as state.

## UI Conventions

Use the components in `crates/ui/src/components` for shared controls, and edit `crates/ui/src/theme/metrics.rs` for their common dimensions. Colors come from the semantic palette in `crates/ui/src/theme.rs`.

Commands and single-line inputs use a 32px height; command labels use GPUI Kit's 12px `XSmall` size. Icon buttons are 30px square. Controls have a 4px radius, the main panel 12px, and arrowless tooltips 10px. Toolbar buttons keep 48px SVG artwork in fixed 80px-high slots with 66-72px widths. Page-tool contraction animates width and spacing while preserving the active page and respecting reduced motion.

The start page places the logo and name on the left, separated from the open/create commands and scrollable history by a vertical rule. History rows show the filename above its directory. A subtle grip at the center of the title bar marks a native window-drag area. Start-page and grip dimensions are grouped under `home` and `titlebar` in the shared metrics. These layout changes have source and formatting checks only; native rendering, dragging, display scaling, and small-window behavior still require a requested build and runtime verification.

Title-bar dropdown menus align to their own trigger's left edge, using the component's measured bounds and window-edge clamping. This positioning change has not been verified in the native UI.

Keep the background, inset content panel, and colored toolbar artwork as the visual identity. Reuse the pinned GPUI Kit APIs and shared components when adding views. Keep asset provenance in [THIRD_PARTY.md](THIRD_PARTY.md). Normal builds use committed assets and do not require Node.js or Python; brand regeneration uses `tools/render-brand.cjs` with Sharp and `tools/generate-icon.py` with Pillow.

## Scope and Verification

The product target is the user-facing functionality of 7-Zip 26.03 File Manager. Current source coverage is partial:

| Area | Current coverage and gaps |
| --- | --- |
| File and Edit | Archive creation, extraction, editing, properties, comments, and recent locations. Full local filesystem management and integrated viewing/editing remain incomplete. |
| View and Favorites | Folder/archive navigation, search, and list sorting are present. The full View command set, alternate layouts, and Favorites are not implemented. |
| Tools and options | Integrity checks, checksums, and general/advanced/appearance/association settings are present. Benchmarks and the complete upstream option/dialog matrix remain incomplete. |
| Help | Bundled 7-Zip help, application information, and optional release checks. |
| Windows integration | Explorer commands, per-user registration, installer maintenance, and single-instance forwarding are implemented; runtime coverage still needs verification across Windows versions. |

This source reorganization has formatting and static source checks only. It has not been compiled or verified in the native UI. In particular, toolbar contraction, focus behavior, display scaling, and translated layouts require runtime verification. Builds are performed only when explicitly requested; generated files in `bin/` are not source validation evidence.

## Third-Party Software

See [THIRD_PARTY.md](THIRD_PARTY.md) for upstream components and artwork provenance. Retain the bundled 7-Zip license files and `assets/fluent/LICENSE` when distributing the application.
