# Project Rules

- Do not proactively build or compile. Run tools/build.ps1 only when the user explicitly requests a build; otherwise leave bin artifacts unchanged.
- Do not add tests, smoke tests, or other test code.
- Keep implementations direct. Do not add speculative compatibility layers.
- Avoid redundant validation and excessive assertions.
- Keep changes scoped and commit complete, reviewable changes.
- Do not modify F:/Workspace/7zplus or F:/Workspace/cardo from this project.

## Engineering Conventions

- Product copy belongs in locales/zh-CN.ftl, zh-TW.ftl and en-US.ftl. Keep semantic keys and parameter names aligned. Use complete locale messages for variable text.
- Do not use translated labels as state, identifiers or branching conditions. Use enums for domain and platform actions.
- Keep archive operations independent of GPUI and registry code. Platform modules emit commands; ui/system.rs dispatches them into the workspace.
- Explorer commands receive the complete IShellItemArray. Keep the Explorer DLL independent of GPUI and the 7z.exe adapter; share only action definitions, format routing and request data through cardo-7zp-commands.
- Context menus must use the clicked target and a selection snapshot. Unavailable operations are omitted; do not clear a multi-selection merely because a background or keyboard menu was opened.
- Use GPUI Kit's matching reexports, Entity/WeakEntity, owned subscriptions, Root and component focus/menu APIs. Consult the pinned dependency sources before copying newer examples.
- Do not perform filesystem, network, registry or process operations during render. Read metadata and run archive work in the background, then update entities.
- Reuse the prototype's SVG icon assets with provenance and licenses.
- Product scope is all user-facing functionality of the pinned 7-Zip 26.03 File Manager, presented through the existing GPUI layout. The user explicitly superseded earlier exclusions of filesystem management, viewing/editing and benchmarks. Implement real behavior and format-specific availability; do not substitute launching 7zFM for integrated application functionality or label incomplete coverage as complete.
- Keep the capability scope and gaps in README.md and README.zh.md aligned with the official File/Edit/View/Favorites/Tools/Help commands, toolbar operations, option pages, format-dependent dialogs, keyboard shortcuts and Windows integration. UI, context menus and shortcuts must share operations and selection semantics.
- Registry changes must be explicit and limited to the application's own per-user keys and values. Do not replace the user's default archive association.
- Do not fabricate progress, data, successful results or claims of visual parity. Document source implementation separately from runtime evidence.
- Maintain architecture, shared UI conventions, verification limits and known gaps in README.md and README.zh.md when behavior changes. Keep asset provenance and licensing in THIRD_PARTY.md.
