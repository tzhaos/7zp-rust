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
- Switch among Simplified Chinese, Traditional Chinese, and English, and between light and dark themes. Appearance also sets the interface font and type size. The font list shows every installed family, and a comma-separated list can be typed.

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

The last three commands support installation and maintenance. The Explorer menu and the executable share command ids in `crates/cardo-7zp-commands`.

Common shortcuts follow the 7-Zip File Manager where the command exists. `Enter` opens the current item, `Shift+Enter` opens it outside, and `Ctrl+Page Down` opens a folder or archive inside. `F2` renames, `F5` copies, and `F6` moves archive entries; `Delete` deletes them. `Ctrl+R` refreshes the current folder, `Backspace` and `Alt+Up` go to the parent, and `Ctrl+A` selects all entries. `Ctrl+O` opens an archive, `Ctrl+L` focuses the address, `Ctrl+F` searches, `Alt+Left` goes back, `Alt+E` opens extraction options, `Alt+T` checks integrity, and `F1` opens 7zplus help. Menus display additional shortcuts.

## Architecture

| Directory | Responsibility |
| --- | --- |
| `crates/cardo-7zp-engine` | Runs the bundled `7z.exe`. It owns archive listings, creation, editing, comments, checksums, and progress parsing. It does not draw a window or write the registry. |
| `crates/cardo-7zp-requests` | The current folder and the background requests the window submits: open, extract, create, read a directory, and check for a release. This is not the executable. |
| `crates/cardo-7zp-platform` | Windows file opening and icons, single-instance commands, registry integration, mail, and installation maintenance. |
| `crates/cardo-7zp-core` | Settings and localization: preferences, appearance, recent locations, language, theme, and the saved extract destination. |
| `crates/cardo-7zp-ui/src/lib.rs` | Workspace state, subscriptions, initialization, and input/focus synchronization. |
| `crates/cardo-7zp-ui/src/actions` | Task lifecycle and workspace coordination for archive operations and preferences. |
| `crates/cardo-7zp-ui/src/views` | Window chrome, menu bar, file lists, and settings-page composition. |
| `crates/cardo-7zp-ui/src/dialogs` | Dialog composition, archive creation form, and error presentation. |
| `crates/cardo-7zp-ui/src/components` | Shared buttons, inputs, tooltips, navigation, lists, settings rows, icons, and menu styling. |
| `crates/cardo-7zp-ui/src/theme.rs`, `crates/cardo-7zp-ui/src/theme` | Shared dimensions, semantic colors, and GPUI theme application. The theme module applies a given theme and does not open settings files. |
| `crates/cardo-7zp-app` | Process entry. The executable name remains `7zplus`. |
| `crates/cardo-7zp-commands` | The command list, archive-type ids, and selected-file payload shared by the Explorer menu and `7zplus`. It has no window and does not call `7z.exe`. |
| `crates/cardo-7zp-explorer` | The Explorer right-click DLL. It writes one request and lets `7zplus.exe` perform it. The installed file is still `7-zip-plus.dll`. |
| `locales`, `assets` | Product text and embedded visual resources. |
| `tools`, `packaging` | Reproducible engine preparation, build scripts, and the NSIS installer. |

UI commands dispatch background work and consume outcomes. Views compose components; they do not perform filesystem, process, network, or registry work. Context menus retain the clicked target and selection snapshot. Product text belongs in all three locale files, using semantic keys rather than translated labels as state.

## UI Conventions

Use the components in `crates/cardo-7zp-ui/src/components` for shared controls, and edit `crates/cardo-7zp-ui/src/theme/metrics.rs` for their common dimensions. Colors come from the semantic palette in `crates/cardo-7zp-ui/src/theme.rs`.

Commands and single-line inputs use a 32px height; command labels use GPUI Kit's 12px `XSmall` size. Icon buttons are 30px square. Controls have a 4px radius, the main panel 12px, and arrowless tooltips 10px. Toolbar buttons keep 48px artwork in fixed 80px-high slots with 66-72px widths. The artwork is rasterized for the current display scale and supersampled before it is drawn. Page tools share one width animation of about 180ms when they collapse or expand. Its final stretch moves at a constant speed instead of crawling toward the end. Switching views does not change the group's width, so the collapse button stays put, and the subpage title stays above the clipped area the new page slides through. The animation respects reduced motion, and frames follow the display refresh. The appearance font and type size apply to the window, file list, start-page history, and settings labels. A comma-separated list is used in order: the first installed family is the face, and the other installed names fill missing glyphs. A name the system does not list stays in the field but is not given to layout. Toolbar slots, the status bar, and command buttons keep those fixed metrics.

The start page places the logo and name on the left, separated from the open/create commands and scrollable history by a vertical rule. History rows show the filename above its directory. A subtle grip at the center of the title bar marks a native window-drag area. Start-page and grip dimensions are grouped under `home` and `titlebar` in the shared metrics. These layout changes have source and formatting checks only; native rendering, dragging, display scaling, and small-window behavior still require a requested build and runtime verification.

Dropdown menus open from the trigger's top-left. Nested menus use the same 280px width as the parent and scroll inside that width; a wider child is what pulls the menu off the item that opened it. Paths are truncated instead of forcing a wider row. This positioning change has not been verified in the native UI. Appearance can hide the text under the large toolbar buttons.

Dialogs share one movable window: a drag title, a scrolling body, and a bottom action bar. Close, Cancel, and Esc use the same dismiss path. The progress window has no close button; it only stops the task. If the window cannot be opened, the same content falls back to a centered card. Help is 7zplus's own text, not the bundled 7-Zip manual. These window changes have not been verified in the native UI.

A scrolling region shows a scrollbar when its content overflows. The track is transparent, the thumb uses the muted text color and the text color on hover, and its radius matches the other controls. It stays hidden when nothing overflows. The file list draws that thumb over the right edge of the list. A search filter uses the same file row as an unfiltered list and does not add a path under the name. File icons carry at most four uppercase extension characters at the bottom-right; directories do not. The scrollbar and extension mark have not been verified in the native UI.

Keep the background, inset content panel, and colored toolbar artwork as the visual identity. Reuse the pinned GPUI Kit APIs and shared components when adding views. Keep asset provenance in [THIRD_PARTY.md](THIRD_PARTY.md). Normal builds use committed assets and do not require Node.js or Python; brand regeneration uses `tools/render-brand.cjs` with Sharp and `tools/generate-icon.py` with Pillow.

## Scope and Verification

The product target is the user-facing functionality of 7-Zip 26.03 File Manager. Current source coverage is partial:

| Area | Current coverage and gaps |
| --- | --- |
| File and Edit | File keeps the File Manager commands that already work on the current item: open, open inside, open outside, rename, copy, move, delete, properties, comment, and checksum, then Exit. Copy, move, rename, and delete still change archive entries only. Edit selects, clears, inverts, and matches the current item's type. Filesystem copy or delete, and an integrated viewer or editor, are not menu entries. |
| View and Favorites | View sorts by name, date, or size, moves up one level, and refreshes. Folder and archive navigation, search, and list sorting remain. Icon layouts, flat view, two panels, and Favorites are not menu entries. |
| Archive | Open, create, reopen by format, recent archives, extract, integrity check, save a copy, archive information, and close live in the Archive menu. The old File entry that only switched back to the file page is gone; that switch stays on the toolbar. |
| Tools and options | Tools opens the existing option pages. Checksums stay in the File menu. Appearance stores the font and type size. The font may be several names separated by commas. Quick extraction can close the open archive afterward; that option is off by default. Extract asks about each existing file instead of choosing an overwrite mode in advance. Benchmarks and the complete upstream option matrix remain incomplete. |
| Help | 7zplus help, application information, and optional release checks. The bundled 7-Zip manual is not opened from the Help menu. |
| Windows integration | Explorer commands, per-user registration, installer maintenance, and single-instance forwarding are implemented; runtime coverage still needs verification across Windows versions. |

This layering type-checks with `cargo check --workspace`. It has not been launched in the native UI. Toolbar contraction, focus behavior, display scaling, and translated layouts still require runtime verification. Builds are performed only when explicitly requested; generated files in `bin/` are not source validation evidence.

## Third-Party Software

See [THIRD_PARTY.md](THIRD_PARTY.md) for upstream components and artwork provenance. Retain the bundled 7-Zip license files and `assets/fluent/LICENSE` when distributing the application.
