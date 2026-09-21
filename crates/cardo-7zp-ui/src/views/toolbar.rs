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
        // Reserve the left actions, toolbar padding/gaps, toggle and divider.
        let fixed_width = 4. * TOOL_MIN_WIDTH + 2. * TOOLBAR_PADDING + 8. * TOOL_GAP + 28.;
        let item_width = ((f32::from(window.viewport_size().width) - fixed_width) / 5. - TOOL_GAP)
            .clamp(TOOL_MIN_WIDTH, TOOL_MAX_WIDTH);
        h_flex()
            .relative()
            .h(px(TOOLBAR_HEIGHT))
            .flex_shrink_0()
            .px(px(TOOLBAR_PADDING))
            .gap(px(TOOL_GAP))
            .items_center()
            .bg(rgb(p.panel))
            .child(
                tool("open", ToolIcon::Open, tr("open"), blocked, cx).on_click(cx.listener(
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
                    cx,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.quick_extract(!this.browser.view().selected.is_empty(), cx)
                })),
            )
            .child(
                tool("check", ToolIcon::Check, tr("check"), inactive, cx).on_click(cx.listener(
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
                h_flex().h(px(TOOL_HEIGHT)).flex_shrink_0().children(
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
                        let openness = gpui_kit::base::spring(
                            (id, "toolbar-width"),
                            if visible { 1.0_f32 } else { 0.0_f32 },
                            gpui_kit::base::Spring::new(std::time::Duration::from_millis(300)),
                            window,
                            cx,
                        );
                        let label = match page {
                            Page::Files => tr("files-view"),
                            Page::Settings(preferences::Tab::Associations) => {
                                tr("settings-associations-short")
                            }
                            Page::Settings(tab) => tab.label(),
                        };
                        let button = tool(id, icon, label, blocked, cx)
                            .disabled(blocked || !visible)
                            .relative()
                            .flex_none()
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
                            .h(px(TOOL_HEIGHT))
                            .w(px((item_width + TOOL_GAP) * openness))
                            .flex_shrink_0()
                            .overflow_hidden()
                            .when(openness > 0., |el| el.child(button))
                    }),
                ),
            )
    }
}
