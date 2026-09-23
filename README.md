<p align="center">
  <img src="assets/brand/logo.svg" width="112" alt="Plus7z">
</p>
<h1 align="center">Plus7z</h1>
<p align="center">A native Windows archive manager, built with Rust and GPUI Kit.</p>
<p align="center">
  <a href="https://github.com/tzhaos/7zp-rust/releases/latest">Download</a> ·
  <a href="README.zh.md">简体中文</a> ·
  <a href="docs/reference.md">Documentation</a> ·
  <a href="https://github.com/tzhaos/7zp-rust/issues">Report an issue</a>
</p>
<p align="center"><strong>Windows x64</strong> · 7-Zip 26.03 · <a href="LICENSE">MIT</a></p>

## Download

Choose an **[installer or portable ZIP](https://github.com/tzhaos/7zp-rust/releases/latest)**. Both include 7-Zip 26.03 and the required Visual C++ runtime.

| Package | Use it for |
| --- | --- |
| `p7z-amd64-installer.exe` | Per-user installation with Explorer integration |
| `p7z-amd64-portable.zip` | Extract the whole folder and run `p7z.exe` |

Releases include `SHA256SUMS.txt`. Installed and portable copies can check for updates in **About**. Settings and history live in `%LOCALAPPDATA%\7zplus-rust`.

## Features

- **Browse** folders and archives with search, sorting, multiple selection and history.
- **Create and extract** 7z, ZIP, TAR, gzip, bzip2, xz and WIM, with options suited to each format.
- **Work with archive entries**: add, rename, delete, copy or move out, and edit ZIP comments where supported.
- **Handle tasks** with conflict choices, integrity checks, cancellation and brief extraction completion notifications.
- **Make it yours**: three languages, light and dark themes, custom fonts and 33 configurable shortcuts.
- **Use Explorer integration** through explicit per-user registration that preserves default file associations.

This is an early implementation. Full filesystem management, integrated viewers/editors, nested archives, additional views, favorites and benchmarks are still incomplete. External edits are not written back to archives. See [scope and verification](docs/reference.md#scope-and-verification).

## Build

Windows x64 · Visual Studio C++ Build Tools + Windows SDK · PowerShell · Rust 1.95.0

```powershell
git clone --recurse-submodules https://github.com/tzhaos/7zp-rust.git
cd 7zp-rust
./tools/build.ps1 -ReleaseRepository tzhaos/7zp-rust -Package All
./bin/p7z.exe
```

For an existing checkout, run `git submodule update --init --recursive`. `-Package Run` builds the runnable folder; `All` also produces the installer and portable ZIP in `dist/`.

The application package is `p7z`; supporting crates use `zip-*`. Shared `cardo-runtime` and `cardo-ui` infrastructure comes from the pinned [Cardo](https://github.com/tzhaos/cardo-rust) submodule. [Architecture, shortcuts and release workflow →](docs/reference.md)

## License

[MIT](LICENSE). Bundled 7-Zip, Fluent icons and the Visual C++ runtime retain their respective licenses. See [third-party notices](THIRD_PARTY.md).
