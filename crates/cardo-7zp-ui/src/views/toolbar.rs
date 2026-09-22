use crate::theme::metrics::*;
use crate::*;
use gpui_kit::base::ElementExt;
use gpui_kit::{
    component::{Disableable, Selectable, button::ButtonVariants, h_flex},
    prelude::FluentBuilder,
};
use cardo_7zp_core::i18n::tr;
use std::{cell::Cell, rc::Rc};

impl Workspace {
    pub(crate) fn toolbar(
        &self,
        active_center: Rc<Cell<Pixels>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let p = crate::theme::palette(cx);
        let blocked = self.tasks.is_busy()
            || self.dialogs.is_open()
            || self.settings_busy(cx)
            || self.preferences_task.is_some();
        let inactive = blocked || self.browser.view().catalog.is_none() || self.page != Page::Files;
        let show_labels = !self.preferences.hide_tool_labels;
        let tool_height = if show_labels {
            TOOL_HEIGHT
        } else {
            TOOL_ICON_HEIGHT
        };
        let toolbar_height = if show_labels {
            TOOLBAR_HEIGHT
        } else {
            TOOLBAR_ICON_HEIGHT
        };
        // Reserve the left actions, toolbar padding/gaps, toggle and divider.
        let fixed_width = 4. * TOOL_MIN_WIDTH + 2. * TOOLBAR_PADDING + 8. * TOOL_GAP + 28.;
        let item_width = ((f32::from(window.viewport_size().width) - fixed_width) / 5. - TOOL_GAP)
            .clamp(TOOL_MIN_WIDTH, TOOL_MAX_WIDTH);
        // One shared width for every non-current page tool. Separate springs do
        // not stay complementary, so the collapse button and the open tool shift
        // back and forth while a view changes.
        let slot = item_width + TOOL_GAP;
        let step = 1.0 / (slot * window.scale_factor()).max(1.0);
        let expand = presented_openness(
            gpui_kit::base::transition(
                "toolbar-expand",
                if self.tools_expanded { 1.0 } else { 0.0 },
                gpui_kit::base::Transition::new(std::time::Duration::from_millis(180))
                    .ease(motion_ease),
                window,
                cx,
            ),
            if self.tools_expanded { 1.0 } else { 0.0 },
            step,
        );
        h_flex()
            .relative()
            .h(px(toolbar_height))
            .flex_shrink_0()
            .px(px(TOOLBAR_PADDING))
            .gap(px(TOOL_GAP))
            .items_center()
            .bg(rgb(p.panel))
            .child(
                tool("open", ToolIcon::Open, tr("open"), blocked, show_labels, window, cx).on_click(cx.listener(
                    |this, _, window, cx| {
                        this.show_page(Page::Files, window, cx);
                        this.open(cx);
                    },
                )),
            )
            .child(
                tool(
                    "extract-options",
                    ToolIcon::Extract,
                    tr("extract-options"),
                    inactive,
                    show_labels,
                    window,
                    cx,
                )
                .on_click(cx.listener(|this, _, window, cx| this.extract_dialog(window, cx))),
            )
            .child(
                tool(
                    "extract",
                    ToolIcon::QuickExtract,
                    if self.browser.view().selected.is_empty() {
                        tr("extract-quick")
                    } else {
                        tr("extract-selected")
                    },
                    inactive,
                    show_labels,
                    window,
                    cx,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.quick_extract(!this.browser.view().selected.is_empty(), cx)
                })),
            )
            .child(
                tool("check", ToolIcon::Check, tr("check"), inactive, show_labels, window, cx).on_click(cx.listener(
                    |this, _, _, cx| {
                        if let Some(catalog) = this.browser.view().catalog.clone() {
                            this.execute(
                                Request::Check(catalog),
                                this.browser.view().password.clone(),
                                cx,
                            );
                        }
                    },
                )),
            )
            .child(div().flex_1())
            .child(
                icon_button(
                    "toolbar-toggle",
                    if self.tools_expanded {
                        "ArrowRight"
                    } else {
                        "ArrowLeft"
                    },
                    tr(if self.tools_expanded {
                        "toolbar-collapse"
                    } else {
                        "toolbar-expand"
                    }),
                    self.allow_hint(true),
                    cx,
                )
                .custom(subtle_variant(cx))
                .border_0()
                .w(px(28.))
                .h(px(30.))
                .flex_shrink_0()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.tools_expanded = !this.tools_expanded;
                    cx.notify();
                })),
            )
            .child(
                div()
                    .w(px(1.))
                    .h(px(32.))
                    .flex_shrink_0()
                    .mx(px(2.))
                    .bg(rgb(p.border)),
            )
            .child(
                h_flex().h(px(tool_height)).flex_shrink_0().children(
                    [
                        ("files", ToolIcon::Files, Page::Files),
                        (
                            "general",
                            ToolIcon::General,
                            Page::Settings(preferences::Tab::General),
                        ),
                        (
                            "appearance",
                            ToolIcon::Appearance,
                            Page::Settings(preferences::Tab::Appearance),
                        ),
                        (
                            "associations",
                            ToolIcon::Associations,
                            Page::Settings(preferences::Tab::Associations),
                        ),
                        (
                            "advanced",
                            ToolIcon::Advanced,
                            Page::Settings(preferences::Tab::Advanced),
                        ),
                    ]
                    .into_iter()
                    .map(|(id, icon, page)| {
                        let visible = self.tools_expanded || self.page == page;
                        // The current page keeps a full slot. Other pages follow the
                        // single expansion value, so a view change does not resize
                        // the group or move the collapse button.
                        let openness = if self.page == page { 1.0 } else { expand };
                        let label = match page {
                            Page::Files => tr("files-view"),
                            Page::Settings(preferences::Tab::Associations) => {
                                tr("settings-associations-short")
                            }
                            Page::Settings(tab) => tab.label(),
                        };
                        let button = tool(id, icon, label, blocked, show_labels, window, cx)
                            .disabled(blocked || !visible)
                            .relative()
                            .flex_none()
                            .opacity(openness)
                            .w(px(item_width))
                            .selected(self.page == page)
                            .when(self.page == page, |el| {
                                let center = active_center.clone();
                                el.on_prepaint(move |bounds, _, _| center.set(bounds.center().x))
                            })
                            .on_click(cx.listener(move |this, _, window, cx| match page {
                                Page::Files => this.show_page(page, window, cx),
                                Page::Settings(tab) => this.preferences_category(tab, window, cx),
                            }));
                        div()
                            .id(("toolbar-slot", page.order() as u32))
                            .h(px(tool_height))
                            .w(px(slot * openness))
                            .flex_shrink_0()
                            .overflow_hidden()
                            .when(openness > 0., |el| el.child(button))
                    }),
                ),
            )
    }
}

fn presented_openness(value: f32, target: f32, step: f32) -> f32 {
    if (value - target).abs() <= step {
        target
    } else {
        (value / step).round() * step
    }
}

/// Fast at the start, then linear through the final distance.
///
/// A critically damped spring only approaches its target, so the last pixels
/// arrive further and further apart and the toolbar looks like it is stuttering.
pub(super) fn motion_ease(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    const KNEE: f32 = 0.62;
    const COVERED: f32 = 0.78;
    if progress < KNEE {
        let unit = progress / KNEE;
        (1.0 - (1.0 - unit) * (1.0 - unit)) * COVERED
    } else {
        COVERED + (1.0 - COVERED) * ((progress - KNEE) / (1.0 - KNEE))
    }
}
