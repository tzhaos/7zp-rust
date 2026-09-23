# 7zplus

[简体中文](README_zh.md) · [Download](https://github.com/tzhaos/7zp-rust/releases/latest)

<img src="assets/brand/logo.svg" alt="7zplus" width="80" />

A native Windows archive manager built with Rust and GPUI Kit, powered by the bundled 7-Zip 26.03 command-line engine.

## Download

Get the latest version from [GitHub Releases](https://github.com/tzhaos/7zp-rust/releases/latest).

| File | Usage |
| --- | --- |
| `7zplus-amd64-installer.exe` | Per-user installation with Explorer integration |
| `7zplus-amd64-portable.zip` | Extract the complete folder and run `7zplus.exe` |
| `SHA256SUMS.txt` | SHA-256 hashes for both packages |

Both packages include the engine and required Visual C++ runtime. Portable means installation-free; configuration and history still use the per-user settings directory.

## Features

- Browse folders and archives with search, sorting, multi-selection and history.
- Create 7z, ZIP, TAR, gzip, bzip2, xz and WIM archives with format-dependent options.
- Extract all or selected entries, resolve filename conflicts, check integrity and cancel tasks.
- Add, rename, delete, copy out and move out archive entries where supported; read/write ZIP comments.
- Open extracted files with their associated Windows applications.
- Per-user Explorer integration and explicit file-type registration without replacing default associations.
- Simplified Chinese, Traditional Chinese and English; light, dark and system themes.
- Configurable fonts, text size and 33 keyboard shortcuts.

Operations use the bundled `7z.exe`, not the official File Manager UI. External edits to extracted files are not automatically written back into the archive.

## Menus and Shortcuts

| Menu | Commands |
| --- | --- |
| File | Open, Save as, archive history, Clear history, Exit |
| Operations | Extract to, Add, archive Properties, Comment, Test |
| Tools | Application, System, Advanced, About |

Sorting and selection belong to file-table headers; navigation and Refresh belong to the path bar. Creation is available on Home and in folder context menus.

Customize shortcuts in **Application > Keyboard shortcuts**: search commands, record or remove bindings, detect conflicts, and restore individual or all defaults. Menu labels follow saved bindings.

| Default shortcut | Action |
| --- | --- |
| Ctrl+O / Ctrl+N / Ctrl+Shift+S | Open / Create / Save archive as |
| Alt+E / Alt+A / Alt+T | Extract to / Add / Test |
| Alt+I / Alt+M | Archive properties / Comment |
| F5 / F2 / Delete | Refresh / Rename / Delete |
| Ctrl+Shift+C / Ctrl+Shift+M | Copy to / Move to |
| Ctrl+L / Ctrl+F | Address bar / Search |
| Ctrl+A / Ctrl+I | Select all / Invert selection |
| Alt+Left / Alt+Up | Back / Parent folder |
| Ctrl+, | Application settings |
| Ctrl+Alt+S / Ctrl+Alt+A / Ctrl+Alt+I | System / Advanced / About |

Enter, Shift+Enter, Ctrl+Page Down, arrows, Space, Backspace, Escape and Shift+F10 retain fixed navigation behavior. Editing shortcuts keep their input-specific scope.

## Build

Requirements: Windows x64, Visual Studio C++ Build Tools with Windows SDK, PowerShell, and the Rust toolchain pinned in `rust-toolchain.toml` (currently 1.95.0). Initial downloads require network access.

Clone with `git clone --recurse-submodules`, or run `git submodule update --init --recursive` in an existing checkout. The `cardo/` submodule contains the shared crates; the application crates under `crates/` are maintained here.

```powershell
./tools/build.ps1 -ReleaseRepository tzhaos/7zp-rust -Package All
./bin/7zplus.exe
```

`-Package Run` builds the runnable directory; `Portable` adds the ZIP, `Installer` adds the installer, and `All` produces both packages. Outputs are in `bin/` and `dist/`. Keep `runtime/7zip/` beside the executable.

## Updates and Releases

About checks the latest stable GitHub release; startup checks are optional. **Download and install** detects whether the running copy is registered as an installation, then downloads the matching installer or portable ZIP. The download is cancellable. 7zplus verifies its SHA-256 against the release manifest, prepares the update, exits, applies it with a separate helper process, and restarts. The helper backs up the files it replaces and restores them if installation fails; an interrupted update is recovered on the next launch. A portable copy needs write access to its directory. Builds without a release repository can still run, but cannot check for updates.

Use `./tools/version.ps1 -Part Patch` (or `Minor` / `Major`) to advance the workspace version and synchronize Cargo.lock. Commit both files before creating and pushing a matching `vX.Y.Z` tag.

The Windows workflow builds branches and pull requests. Matching tags publish packages after verification; manual dispatch with an existing matching tag creates a draft. Internal `dist/build-info.json` records version, commit, repository, target, working-tree state and checksum provenance. Publishing verifies uploaded packages by downloading them before making the release public; reruns preserve published assets.

## Settings and Diagnostics

Settings and history live in `%LOCALAPPDATA%\7zplus-rust`. Writes are atomic. Missing configuration uses defaults; unreadable or malformed files produce explicit errors. Fonts are validated against installed families instead of silently replacing missing names.

Logs in the `logs/` subdirectory rotate daily in UTC and retain up to 14 files. They record operation failures and error chains; command arguments and passwords are not logged by the application.

## Architecture

| Crate | Responsibility |
| --- | --- |
| `zip-app` | Startup and packaging |
| `zip-ui` | Workspace, settings, dialogs and command dispatch |
| `zip-core` | Preferences, localization, history and shortcut definitions |
| `zip-engine` | 7-Zip adapter, operations and progress |
| `zip-requests` | Background requests, release checks and update downloads |
| `zip-platform` | Windows integration, registry policy, update application and single instance |
| `zip-explorer` | Explorer extension, independent of GPUI and the engine adapter |
| `zip-commands` | Shared command identities, routing and request data |
| `cardo-ui` | Reusable settings, menus, tooltips, text, font and theme components |
| `cardo-runtime` | Atomic storage, Fluent catalogs, diagnostics and registry ownership |

The `cardo-*` crates live in the separate [Cardo repository](https://github.com/tzhaos/cardo-rust) at the commit pinned by the submodule. This repository owns the `zip-*` crates and its own release version.

Business work runs outside render. Product modules own values and persistence; shared components own presentation. Dialogs share frames and artwork notices. Command menus are native Windows surfaces; settings selectors remain themed. Constrained text uses ellipsis with full-text hints only when truncated.

## Scope and Verification

The target is the official 7-Zip 26.03 File Manager's user-facing functionality. This early implementation does not yet provide complete parity.

| Area | Limitations |
| --- | --- |
| File/Edit | Editing primarily targets archive entries; full filesystem management and integrated viewers/editors are incomplete. |
| View/Favorites | Nested archive navigation, flat/icon views, dual panels and favorites are incomplete. |
| Tools/Options | Benchmarks and the complete upstream option set are not implemented. |
| Help | About includes version and updates; no integrated help page. |
| Windows | Integration exists; all OS versions, ownership conflicts and interrupted installations are not fully verified. |

Local builds and selected engine operations have been verified, including v0.2.1's input-list fix for 7z/ZIP creation, updates, deletion and checksum generation. This does not establish full GUI, installation/upgrade, cancellation, accessibility, DPI or multi-monitor coverage. Some native-window lifecycle/accessibility diagnostics remain unresolved.

## Third-Party Components

The engine is [7-Zip](https://www.7-zip.org/); retain its included license and notices. Small icons come from [Microsoft Fluent UI System Icons](https://github.com/microsoft/fluentui-system-icons), with their [MIT license](assets/fluent/LICENSE). Brand and toolbar artwork are original project assets. The Visual C++ runtime is distributed under Microsoft's redistribution terms.

The project source is licensed under [MIT](LICENSE). The Cardo submodule carries its own MIT license.
