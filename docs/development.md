# UI Development

The file manager is a Cargo workspace. `src/main.rs` is the binary. Archive models and the 7-Zip adapter live in `crates/archive`, background requests in `crates/application`, Windows integration in `crates/platform`, and settings plus Fluent in `crates/core`. `crates/ui` holds actions, views, dialogs, components, and theme. The theme module applies a supplied theme and does not open settings files.

The start page is implemented in `crates/ui/src/views/home.rs`; file and history rows are in `views/files.rs`, and native window chrome is in `views/titlebar.rs`. Shared dimensions live in `crates/ui/src/theme/metrics.rs`, with dedicated `home` and `titlebar` groups. Command and input height 32, icon button size 30, radii 4 / 12 / 10, toolbar artwork 48, tool height 80, and tool width 66–72 are defined once in that module and used by the shared controls.

The start page uses the same `Command::Open` and `Command::Create` dispatch as the menu bar. History keeps its existing open, remove, and clicked-target context-menu behavior. The title-bar grip uses GPUI's `WindowControlArea::Drag` and does not introduce a separate mouse-drag state machine.

Title-bar menus use `DropdownMenu::dropdown_menu` with its default left anchor. A right anchor makes menus wider than their triggers extend beyond the left window edge and clamp to the same position. The component continues to own trigger measurement and viewport clamping.

Preferences, recent locations, language, theme, and the saved extract destination are persisted by `crates/core`. The workspace layout and that settings split are in place.

No build or native runtime verification was performed for the start-page changes. Existing `bin/` files represent earlier source. Builds remain opt-in; no test code was added.
