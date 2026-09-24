# v0.3.0 verification

[简体中文](verification-v0.3.0.zh.md) · [Reference](reference.md)

24 September 2026, Windows x64. No test or smoke-test code was added. Work took place in this checkout and its Cardo submodule.

## Builds and package inspection

- Each extraction batch passed a Cardo build and `tools/build.ps1` before commit.
- Final version is 0.3.0 in both workspaces. GPUI Kit remains 0.6.1; Cardo Desk keeps its own 0.1.0 application version.
- `cargo check --workspace`, minimal runtime/app feature checks, Cardo all-feature release build and independent template default/update builds passed.
- `tools/build.ps1 -ReleaseRepository tzhaos/7zp-rust -Package All` produced the installer, portable ZIP and checksums.
- Portable ZIP integrity check passed. Its 16 files exactly match the updater whitelist, including `Cardo-dependencies-LICENSE.txt`. Both package SHA-256 values matched `SHA256SUMS.txt`.
- Locale key/parameter checks matched all three languages: 489 product messages, 21 Cardo runtime messages, 21 template messages.
- Cardo crate/template source contains no product identity or `p7z-*` dependency. Replaced native pipe/icon/updater code and duplicated presentation implementation were removed from P7Z; remaining wrappers supply product labels, artwork, sizes and policy.

## Runtime observations

- Home and Application settings rendered in Simplified Chinese. Native Create opened repeatedly and closed using Escape, Cancel and Alt+F4. The main titlebar Close control exited the process.
- English SHA-256 report processed the project LICENSE: 1,067 bytes, `7263f93d0b8e3d71aa30c14f9626566cecb55c92ef95d48318e9d7664f257086`, matching the OS hash tool.
- A second launch passed `--extract-here` and a local acceptance ZIP to the existing instance, exited 0, and queued work while the report dialog was open. Closing that dialog resumed processing.
- Extraction created the expected file with the same SHA-256. Choosing Rename on conflict created a separate file. The Extract form's input was created/focused in its own native window.
- Skipping the only conflicting entry visibly displayed “Skipped every file that already exists.” as a Toast. Successful extraction visibly displayed “Extraction operation completed” as a Toast. Both later disappeared without a result dialog. The exact expiry interval and replacement race were not instrumented.
- Traditional Chinese Home and Application settings rendered. Language was selected through launch arguments without saving product preferences.
- Cardo Desk independently performed the same file hash, saved Chinese/dark settings, showed and expired a save Toast, rejected an external TOML modification without overwrite, and accepted a second-instance request.

Acceptance files are local under `.tools/acceptance-v030`, outside version control. Opening that ZIP added a normal history entry. The template uses its own `%LOCALAPPDATA%\cardo-desk` settings, never the product's data directory.

## Unverified scenarios

Forced native-window failure, all dialog focus/owner-release races, Toast dragging and viewport limits, shortcut conflict/disable/input-edit combinations, font-error presentation, interrupted settings saves, task cancellation during close, full menu/selection matrix, all themes/locales/DPI combinations and multi-monitor placement remain runtime gaps.

Update cancellation, checksum-failure handling, installer/portable application and interrupted recovery/rollback were reviewed in source but **not executed in an isolated installation for this release**. The user's installed application and registration were not used for update acceptance. Package creation is not installation/update verification.

Existing GPUI window/accessibility warnings remain a known diagnostic limitation. Full 7-Zip File Manager parity remains incomplete as documented in the reference. Tags are pushed after local validation; remote builds are not waited on or rechecked.
