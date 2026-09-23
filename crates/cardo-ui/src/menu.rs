use gpui_kit::base::{Align, ElementExt as _, Placement, Positioner};
use gpui_kit::{
    component::{
        ActiveTheme,
        button::Button,
        h_flex,
        menu::{PopupMenu, PopupMenuItem},
    },
    prelude::FluentBuilder,
    *,
};
use std::{cell::Cell, collections::HashMap, rc::Rc};

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Default)]
struct MenuHosts(HashMap<WindowId, WeakEntity<MenuHost>>);
impl Global for MenuHosts {}

/// Windows own their menus and subscriptions; the registry only holds weak references.
pub struct MenuHost {
    menu: Option<Entity<PopupMenu>>,
    anchor: Bounds<Pixels>,
    viewport: Size<Pixels>,
    previous_focus: Option<FocusHandle>,
    more_label: fn() -> SharedString,
    dismiss_subscription: Option<Subscription>,
    _activation_subscription: Subscription,
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
            previous_focus: None,
            more_label,
            dismiss_subscription: None,
            _activation_subscription: cx.observe_window_activation(window, |this, window, cx| {
                if !window.is_window_active() {
                    this.close(window, cx);
                }
            }),
        });
        if !cx.has_global::<MenuHosts>() {
            cx.set_global(MenuHosts::default());
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
            .whitespace_nowrap()
            .text_ellipsis()
            .children(self.menu.as_ref().map(|menu| {
                deferred(
                    Positioner::side(self.anchor)
                        .placement(Placement::Bottom)
                        .align(Align::Start)
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
}

enum Entry {
    Item(MenuItem),
    Submenu(SharedString, Menu),
    Separator,
}

pub struct MenuItem {
    label: SharedString,
    shortcut: Option<&'static str>,
    disabled: bool,
    checked: bool,
    handler: Option<Handler>,
}

impl MenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
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
        self.show_at(Bounds::new(position, size(px(0.), px(0.))), window, cx);
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
        let row_height = (font_size * 1.5 + px(10.)).max(px(32.));
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
                },
            ));
        }
        PopupMenu::build(window, cx, move |mut popup, window, cx| {
            popup = popup.min_w(width).max_w(width).scrollable(false);
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
                        let shortcut = item.shortcut;
                        let disabled = item.disabled || item.handler.is_none();
                        let mut row = PopupMenuItem::element(move |_, cx| {
                            crate::tooltip::bubble_tooltip(
                                h_flex()
                                    .id("menu-label")
                                    .w(width - px(48.))
                                    .h(row_height)
                                    .min_w_0()
                                    .gap(px(12.))
                                    .aria_label(label.clone())
                                    .child(div().flex_1().min_w_0().truncate().child(label.clone()))
                                    .when_some(shortcut, |row, shortcut| {
                                        row.child(
                                            div()
                                                .flex_shrink_0()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(shortcut),
                                        )
                                    }),
                                label.clone(),
                            )
                        })
                        .disabled(disabled)
                        .checked(item.checked);
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

pub trait MenuTrigger {
    fn popup_menu(self, build: impl Fn(&mut Window, &mut App) -> Menu + 'static) -> Self;
    fn choice_menu(self, build: impl Fn(&mut Window, &mut App) -> Menu + 'static) -> Self;
}

impl MenuTrigger for Button {
    fn choice_menu(self, build: impl Fn(&mut Window, &mut App) -> Menu + 'static) -> Self {
        let bounds = Rc::new(Cell::new(Bounds::<Pixels>::default()));
        let measured = bounds.clone();
        self.on_prepaint(move |value, _, _| measured.set(value))
            .on_click(move |_, window, cx| {
                let anchor = bounds.get();
                let mut menu = build(window, cx);
                menu.width = Some(anchor.size.width);
                menu.show_at(anchor, window, cx);
            })
    }

    fn popup_menu(self, build: impl Fn(&mut Window, &mut App) -> Menu + 'static) -> Self {
        let bounds = Rc::new(Cell::new(Bounds::<Pixels>::default()));
        let measured = bounds.clone();
        self.on_prepaint(move |value, _, _| measured.set(value))
            .on_click(move |_, window, cx| build(window, cx).show_at(bounds.get(), window, cx))
    }
}
