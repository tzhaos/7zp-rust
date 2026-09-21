# UI Development

The Cargo workspace migration remains in progress. The start page is implemented in `crates/ui/src/views/home.rs`; file and history rows are in `views/files.rs`, and native window chrome is in `views/titlebar.rs`. Shared dimensions live in `theme/metrics.rs`, with dedicated `home` and `titlebar` groups.

The start page uses the same `Command::Open` and `Command::Create` dispatch as the menu bar. History keeps its existing open, remove, and clicked-target context-menu behavior. The title-bar grip uses GPUI's `WindowControlArea::Drag` and does not introduce a separate mouse-drag state machine.

Title-bar menus use `DropdownMenu::dropdown_menu` with its default left anchor. A right anchor makes menus wider than their triggers extend beyond the left window edge and clamp to the same position. The component continues to own trigger measurement and viewport clamping.

Workspace settings ownership, settings persistence separation, and the remaining local visual constants still need architectural consolidation. The latest layout work does not complete those outstanding refactors.

No build or native runtime verification was performed for the start-page changes. Existing `bin/` files represent earlier source. Builds remain opt-in; no test code was added.
