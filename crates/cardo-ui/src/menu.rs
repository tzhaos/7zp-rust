use gpui_kit::base::{Align, Placement, Positioner};
use gpui_kit::{
    component::{
        ActiveTheme, Icon, Side, h_flex,
        menu::{PopupMenu, PopupMenuItem},
        native_menu::NativeMenu,
        v_flex,
    },
    prelude::FluentBuilder,
    *,
};
use std::{collections::HashMap, rc::Rc};

mod trigger;
pub use trigger::{MenuAnchor, MenuTrigger};

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Default)]
struct MenuHosts(HashMap<WindowId, WeakEntity<MenuHost>>);
impl Global for MenuHosts {}

#[derive(Clone, PartialEq, Action)]
#[action(no_json)]
struct NativeSelection {
    window: AnyWindowHandle,
    epoch: u64,
    index: usize,
}

/// Windows own their menus and subscriptions; the registry only holds weak references.
pub struct MenuHost {
    menu: Option<Entity<PopupMenu>>,
    anchor: Bounds<Pixels>,
    viewport: Size<Pixels>,
    align_end: bool,
    previous_focus: Option<FocusHandle>,
    more_label: fn() -> SharedString,
    dismiss_subscription: Option<Subscription>,
    _activation_subscription: Subscription,
    native_epoch: u64,
    native_handlers: Vec<Handler>,
}

impl MenuHost {
    pub fn is_open(window: &Window, cx: &App) -> bool {
        cx.try_global::<MenuHosts>()
            .and_then(|hosts| hosts.0.get(&window.window_handle().window_id()))
            .and_then(WeakEntity::upgrade)
            .is_some_and(|host| host.read(cx).menu.is_some())
    }

    pub fn install(
        window: &mut Window,
        cx: &mut App,
        more_label: fn() -> SharedString,
    ) -> Entity<Self> {
        let host = cx.new(|cx: &mut Context<Self>| Self {
            menu: None,
            anchor: Bounds::default(),
            viewport: window.viewport_size(),
            align_end: false,
            previous_focus: None,
            more_label,
            dismiss_subscription: None,
            native_epoch: 0,
            native_handlers: Vec::new(),
            _activation_subscription: cx.observe_window_activation(window, |this, window, cx| {
                if !window.is_window_active() {
                    this.close(window, cx);
                }
            }),
        });
        if !cx.has_global::<MenuHosts>() {
            cx.set_global(MenuHosts::default());
            cx.on_action(|selection: &NativeSelection, cx| {
                let window = selection.window;
                let Some(host) = cx
                    .global::<MenuHosts>()
                    .0
                    .get(&window.window_id())
                    .and_then(WeakEntity::upgrade)
                else {
                    return;
                };
                let handler = host.update(cx, |host, _| {
                    if host.native_epoch != selection.epoch {
                        return None;
                    }
                    let handler = host.native_handlers.get(selection.index).cloned();
                    host.native_handlers.clear();
                    handler
                });
                if let Some(handler) = handler {
                    cx.defer(move |cx| {
                        let _ = window.update(cx, |_, window, cx| handler(window, cx));
                    });
                }
            });
        }
        let hosts = &mut cx.global_mut::<MenuHosts>().0;
        hosts.retain(|_, host| host.upgrade().is_some());
        hosts.insert(window.window_handle().window_id(), host.downgrade());
        host
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(menu) = self.menu.take() {
            if menu.focus_handle(cx).contains_focused(window, cx)
                && let Some(focus) = &self.previous_focus
            {
                focus.focus(window, cx);
            }
            self.dismiss_subscription = None;
            self.previous_focus = None;
            cx.notify();
        }
    }

    fn open(
        &mut self,
        menu: Menu,
        anchor: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close(window, cx);
        self.previous_focus = window.focused(cx);
        self.anchor = anchor;
        self.viewport = window.viewport_size();
        self.align_end = menu.align_end;
        let menu = menu.into_popup((self.more_label)(), self.previous_focus.clone(), window, cx);
        self.dismiss_subscription =
            Some(
                cx.subscribe_in(&menu, window, |this, _, _: &DismissEvent, window, cx| {
                    this.close(window, cx);
                }),
            );
        menu.focus_handle(cx).focus(window, cx);
        self.menu = Some(menu);
        cx.notify();
    }
}

impl Render for MenuHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.viewport != window.viewport_size() {
            self.close(window, cx);
        }
        div()
            .absolute()
            .when(self.menu.is_some(), |layer| {
                // The popup occludes this layer; background scrolling still reaches
                // its original scroll owner after dismissing the stale anchor.
                layer
                    .inset_0()
                    .on_scroll_wheel(cx.listener(|this, _, window, cx| this.close(window, cx)))
            })
            .whitespace_nowrap()
            .text_ellipsis()
            .children(self.menu.as_ref().map(|menu| {
                deferred(
                    Positioner::side(self.anchor)
                        .placement(Placement::Bottom)
                        .align(if self.align_end {
                            Align::End
                        } else {
                            Align::Start
                        })
                        .offset(px(6.))
                        .margin(px(8.))
                        .child(menu.clone()),
                )
                .with_priority(gpui_kit::base::POPUP_PRIORITY)
            }))
    }
}

#[derive(Default)]
pub struct Menu {
    entries: Vec<Entry>,
    width: Option<Pixels>,
    align_end: bool,
}

enum Entry {
    Item(MenuItem),
    Submenu(SharedString, Menu),
    Separator,
}

pub struct MenuItem {
    label: SharedString,
    description: Option<SharedString>,
    icon: Option<Icon>,
    shortcut: Option<&'static str>,
    disabled: bool,
    checked: bool,
    handler: Option<Handler>,
}

impl MenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            description: None,
            icon: None,
            shortcut: None,
            disabled: false,
            checked: false,
            handler: None,
        }
    }
    pub fn shortcut(mut self, shortcut: &'static str) -> Self {
        self.shortcut = (!shortcut.is_empty()).then_some(shortcut);
        self
    }
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
    pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.handler = Some(Rc::new(handler));
        self
    }
}

impl Menu {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn item(mut self, item: impl Into<Option<MenuItem>>) -> Self {
        if let Some(item) = item.into() {
            self.entries.push(Entry::Item(item));
        }
        self
    }
    pub fn submenu(mut self, label: impl Into<SharedString>, menu: Menu) -> Self {
        if !menu.entries.is_empty() {
            self.entries.push(Entry::Submenu(label.into(), menu));
        }
        self
    }
    pub fn separator(mut self) -> Self {
        if !self.entries.is_empty() && !matches!(self.entries.last(), Some(Entry::Separator)) {
            self.entries.push(Entry::Separator);
        }
        self
    }
    pub fn show(self, position: Point<Pixels>, window: &mut Window, cx: &mut App) {
        self.show_native(position, window, cx);
    }

    pub fn show_native(self, position: Point<Pixels>, window: &mut Window, cx: &mut App) {
        let host = cx
            .try_global::<MenuHosts>()
            .and_then(|hosts| hosts.0.get(&window.window_handle().window_id()))
            .and_then(WeakEntity::upgrade);
        let Some(host) = host else {
            return;
        };
        let native = host.update(cx, |host, cx| {
            host.close(window, cx);
            host.native_epoch += 1;
            host.native_handlers.clear();
            self.into_native(
                window.window_handle(),
                host.native_epoch,
                &mut host.native_handlers,
            )
        });
        // The pinned implementation runs the OS modal loop after GPUI borrows are released.
        native.show(position, window, cx);
    }

    fn into_native(
        self,
        window: AnyWindowHandle,
        epoch: u64,
        handlers: &mut Vec<Handler>,
    ) -> NativeMenu {
        let mut menu = NativeMenu::new();
        for entry in self.entries {
            menu = match entry {
                Entry::Separator => menu.separator(),
                Entry::Submenu(label, submenu) => {
                    menu.submenu(label, submenu.into_native(window, epoch, handlers))
                }
                Entry::Item(item) => {
                    let label = if let Some(shortcut) = item.shortcut {
                        format!("{}\t{shortcut}", item.label).into()
                    } else {
                        item.label
                    };
                    let disabled = item.disabled || item.handler.is_none();
                    let action = Box::new(NativeSelection {
                        window,
                        epoch,
                        index: handlers.len(),
                    });
                    if let Some(handler) = item.handler {
                        handlers.push(handler);
                    }
                    if disabled {
                        menu.menu_with_disabled(label, true, action)
                    } else if let Some(icon) = item.icon {
                        menu.menu_with_icon(label, icon, action)
                    } else {
                        menu.menu_with_check(label, item.checked, action)
                    }
                }
            };
        }
        menu
    }
    fn show_at(self, anchor: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let host = cx
            .try_global::<MenuHosts>()
            .and_then(|hosts| hosts.0.get(&window.window_handle().window_id()))
            .and_then(WeakEntity::upgrade);
        if let Some(host) = host {
            host.update(cx, |host, cx| host.open(self, anchor, window, cx));
        } else {
            tracing::error!("Cannot show menu without a window menu host");
        }
    }

    fn into_popup(
        mut self,
        more: SharedString,
        focus: Option<FocusHandle>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<PopupMenu> {
        while matches!(self.entries.last(), Some(Entry::Separator)) {
            self.entries.pop();
        }
        let viewport = window.viewport_size();
        let font_size = cx.theme().font_size * 0.875;
        let descriptions = self
            .entries
            .iter()
            .any(|entry| matches!(entry, Entry::Item(item) if item.description.is_some()));
        let icons = self
            .entries
            .iter()
            .any(|entry| matches!(entry, Entry::Item(item) if item.icon.is_some()));
        let row_height = (font_size * if descriptions { 2.8 } else { 1.5 } + px(10.)).max(px(34.));
        let width = self
            .width
            .unwrap_or_else(|| (font_size * 24.).max(px(280.)))
            .min(viewport.width - px(32.));
        let capacity = ((viewport.height - px(40.)).as_f32() / (row_height + px(2.)).as_f32())
            .floor()
            .max(2.) as usize;
        // The pinned component cannot combine scrolling and cascading submenus.
        // Bound every level and expose overflow through another submenu.
        if self.entries.len() > capacity {
            let rest = self.entries.split_off(capacity - 1);
            while matches!(self.entries.last(), Some(Entry::Separator)) {
                self.entries.pop();
            }
            let rest = rest
                .into_iter()
                .skip_while(|entry| matches!(entry, Entry::Separator))
                .collect();
            self.entries.push(Entry::Submenu(
                more.clone(),
                Menu {
                    entries: rest,
                    width: self.width,
                    align_end: self.align_end,
                },
            ));
        }
        PopupMenu::build(window, cx, move |mut popup, window, cx| {
            popup = popup
                .min_w(width)
                .max_w(width)
                .scrollable(false)
                .check_side(Side::Right);
            if let Some(focus) = focus.clone() {
                popup = popup.action_context(focus);
            }
            for entry in self.entries {
                popup = match entry {
                    Entry::Separator => popup.separator(),
                    Entry::Submenu(label, menu) => popup.item(PopupMenuItem::submenu(
                        label,
                        menu.into_popup(more.clone(), focus.clone(), window, cx),
                    )),
                    Entry::Item(item) => {
                        let label = item.label;
                        let description = item.description;
                        let tooltip: SharedString = description
                            .as_ref()
                            .map(|description| format!("{label}\n{description}").into())
                            .unwrap_or_else(|| label.clone());
                        let shortcut = item.shortcut;
                        let disabled = item.disabled || item.handler.is_none();
                        let mut row = PopupMenuItem::element(move |_, cx| {
                            crate::tooltip::bubble_tooltip(
                                h_flex()
                                    .id("menu-label")
                                    .w(width - px(if icons { 72. } else { 48. }))
                                    .h(row_height)
                                    .min_w_0()
                                    .gap(px(12.))
                                    .aria_label(label.clone())
                                    .child(
                                        v_flex()
                                            .flex_1()
                                            .min_w_0()
                                            .gap(px(2.))
                                            .child(div().min_w_0().truncate().child(label.clone()))
                                            .when_some(description.clone(), |el, description| {
                                                el.child(
                                                    div()
                                                        .min_w_0()
                                                        .truncate()
                                                        .text_size(font_size * 0.9)
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(description),
                                                )
                                            }),
                                    )
                                    .when_some(shortcut, |row, shortcut| {
                                        row.child(
                                            div()
                                                .flex_shrink_0()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(shortcut),
                                        )
                                    }),
                                tooltip.clone(),
                            )
                        })
                        .disabled(disabled)
                        .checked(item.checked);
                        if let Some(icon) = item.icon {
                            row = row.icon(icon);
                        }
                        if let Some(handler) = item.handler {
                            row = row.on_click(move |_, window, cx| {
                                let owner = window.window_handle();
                                let handler = handler.clone();
                                // Release entity borrows before invoking product actions.
                                cx.defer(move |cx| {
                                    if let Err(error) = owner.update(cx, |_, window, cx| handler(window, cx)) {
                                        tracing::error!(error = %error, "Cannot dispatch menu action");
                                    }
                                });
                            });
                        }
                        popup.item(row)
                    }
                };
            }
            popup
        })
    }
}
