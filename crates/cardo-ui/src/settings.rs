//! Settings presentation only; callers own values, validation and persistence.
use crate::text::body_text;
use gpui_kit::{
    base::{Radio, RadioGroup, Switch as BaseSwitch, SwitchThumb, SwitchTrack, spring},
    component::{
        ActiveTheme, Sizable,
        button::{Button, ButtonCustomVariant, ButtonVariants},
        checkbox::Checkbox,
        h_flex,
        input::{Input, InputState, Textarea, TextareaState},
        scroll::{Scrollable, ScrollableElement},
        v_flex,
    },
    prelude::FluentBuilder,
    *,
};

pub mod metrics {
    pub const GROUP_RADIUS: f32 = 20.;
    pub const GROUP_PADDING: f32 = 18.;
    pub const ROW_HEIGHT: f32 = 68.;
    pub const ROW_PADDING: f32 = 14.;
    pub const CONTROL_HEIGHT: f32 = 32.;
    pub const CONTROL_RADIUS: f32 = 10.;
    pub const INPUT_WIDTH: f32 = 260.;
    pub const PICKER_SCROLLBAR_GUTTER: f32 = 20.;
}

pub fn frame(cx: &App) -> Div {
    v_flex()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .border_1()
        .border_color(cx.theme().border)
        .rounded(px(metrics::GROUP_RADIUS))
}

pub fn group(rows: impl IntoIterator<Item = AnyElement>, cx: &App) -> Div {
    frame(cx)
        .px(px(metrics::GROUP_PADDING))
        .children(rows.into_iter().enumerate().map(|(index, row)| {
            div()
                .min_w_0()
                .flex_shrink_0()
                .when(index > 0, |el| {
                    el.border_t_1().border_color(cx.theme().border)
                })
                .child(row)
        }))
}

pub fn row(
    label: impl Into<SharedString>,
    description: Option<SharedString>,
    control: impl IntoElement,
    cx: &App,
) -> Div {
    let font_size = cx.theme().font_size * 0.75;
    h_flex()
        .w_full()
        .min_w_0()
        .min_h(px(metrics::ROW_HEIGHT))
        .py(px(metrics::ROW_PADDING))
        .gap(px(20.))
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(px(3.))
                .child(
                    body_text(label)
                        .text_size(font_size + px(2.))
                        .line_height(relative(1.5))
                        .font_weight(FontWeight::SEMIBOLD),
                )
                .when_some(description, |el, description| {
                    el.child(
                        body_text(description)
                            .text_size(font_size)
                            .line_height(relative(1.5))
                            .text_color(cx.theme().muted_foreground),
                    )
                }),
        )
        .child(
            div()
                .min_w_0()
                .max_w(relative(0.52))
                .flex_shrink_0()
                .text_size(font_size + px(1.))
                .child(control),
        )
}

pub fn action(id: impl Into<ElementId>, label: &str, _cx: &App) -> Button {
    Button::new(id)
        .small()
        .label(label.to_owned())
        .accessibility_label(label.to_owned())
        .secondary()
        .h(px(metrics::CONTROL_HEIGHT))
        .min_w_0()
        .max_w_full()
        .px(px(12.))
        .rounded(px(metrics::CONTROL_RADIUS))
        .border_0()
        .shadow_none()
}

pub fn primary_action(id: impl Into<ElementId>, label: &str, cx: &App) -> Button {
    action(id, label, cx).custom(
        ButtonCustomVariant::new(cx)
            .color(cx.theme().foreground)
            .foreground(cx.theme().background)
            .hover(cx.theme().muted_foreground)
            .active(cx.theme().foreground),
    )
}

pub fn choice(id: impl Into<ElementId>, label: &str, cx: &App) -> Button {
    action(id, label, cx)
        .max_w(px(200.))
        .border_1()
        .border_color(cx.theme().input)
        .dropdown_caret(true)
}

pub fn textarea(state: &Entity<TextareaState>, label: &str) -> Textarea {
    Textarea::new(state)
        .aria_label(label.to_owned())
        .h(px(120.))
        .min_w_0()
        .w_full()
        .rounded(px(metrics::CONTROL_RADIUS))
}

pub fn path_value(
    id: impl Into<ElementId>,
    value: impl Into<SharedString>,
    cx: &App,
) -> Stateful<Div> {
    crate::text::compact_text(id, value)
        .text_ellipsis_middle()
        .text_color(cx.theme().muted_foreground)
}

pub fn editor(
    label: &str,
    description: &str,
    action: impl IntoElement,
    input: impl IntoElement,
    cx: &App,
) -> Div {
    v_flex()
        .w_full()
        .min_w_0()
        .pb(px(metrics::ROW_PADDING))
        .child(row(
            label.to_owned(),
            Some(description.to_owned().into()),
            action,
            cx,
        ))
        .child(input)
}

pub fn input(state: &Entity<InputState>, label: &str) -> Input {
    Styled::h(Input::new(state).small(), px(metrics::CONTROL_HEIGHT))
        .aria_label(label.to_owned())
        .min_w_0()
        .max_w_full()
        .rounded(px(metrics::CONTROL_RADIUS))
}

pub fn segments(id: impl Into<ElementId>, label: &str) -> RadioGroup {
    RadioGroup::new(id)
        .axis(Axis::Horizontal)
        .aria_label(label.to_owned())
        .flex()
        .items_center()
        .gap(px(4.))
        .max_w_full()
}

pub fn segment(
    id: impl Into<ElementId>,
    label: &str,
    selected: bool,
    disabled: bool,
    cx: &App,
) -> Radio {
    Radio::new(id)
        .accessibility_label(label.to_owned())
        .checked(selected)
        .disabled(disabled)
        .flex()
        .items_center()
        .justify_center()
        .min_w_0()
        .h(px(30.))
        .px(px(12.))
        .rounded(px(15.))
        .focus_visible(|style| style.shadow(vec![focus_ring(cx)]))
        .text_color(cx.theme().muted_foreground)
        .hover(|style| style.bg(cx.theme().secondary))
        .styles(|styles| {
            styles
                .checked(|style| {
                    style
                        .bg(cx.theme().secondary)
                        .text_color(cx.theme().foreground)
                })
                .disabled(|style| style.opacity(0.5))
        })
        .child(div().min_w_0().truncate().child(label.to_owned()))
}

pub fn stepper(
    value: impl Into<SharedString>,
    decrease: Button,
    increase: Button,
    cx: &App,
) -> Div {
    let value: SharedString = value.into();
    h_flex()
        .w(px(116.))
        .h(px(metrics::CONTROL_HEIGHT))
        .rounded(px(metrics::CONTROL_RADIUS))
        .border_1()
        .border_color(cx.theme().input)
        .child(decrease.w(px(30.)).h(px(30.)).rounded(px(9.)))
        .child(div().flex_1().min_w_0().text_center().child(value))
        .child(increase.w(px(30.)).h(px(30.)).rounded(px(9.)))
}

pub fn picker_surface(cx: &App) -> Div {
    v_flex()
        .min_w_0()
        .p(px(6.))
        .rounded(px(20.))
        .bg(cx.theme().popover)
        .text_color(cx.theme().popover_foreground)
        .border_1()
        .border_color(cx.theme().border)
        .shadow_md()
}

pub fn picker_option(id: impl Into<ElementId>, label: &str, cx: &App) -> Checkbox {
    Checkbox::new(id)
        .accessibility_label(label.to_owned())
        .small()
        .w_full()
        .min_w_0()
        .min_h(px(34.))
        .px(px(10.))
        .py(px(8.))
        .rounded(px(12.))
        .flex_row_reverse()
        .items_center()
        .hover(|style| style.bg(cx.theme().secondary))
        .child(body_text(label.to_owned()).flex_1())
}

pub fn picker_list(id: &'static str, height: Pixels) -> Scrollable<Stateful<Div>> {
    v_flex()
        .id(id)
        .h(height)
        .min_h_0()
        .flex_shrink_0()
        // The pinned scrollbar overlays a 16px track; keep rows outside it.
        .pr(px(metrics::PICKER_SCROLLBAR_GUTTER))
        .gap(px(2.))
        .overflow_y_scrollbar()
        .id(id)
}

#[derive(IntoElement)]
pub struct SettingsSwitch {
    base: BaseSwitch,
    id: ElementId,
    checked: bool,
    disabled: bool,
}

impl SettingsSwitch {
    pub fn new(id: impl Into<ElementId>, label: &str, checked: bool, disabled: bool) -> Self {
        let id = id.into();
        Self {
            base: BaseSwitch::new(id.clone())
                .accessibility_label(label.to_owned())
                .checked(checked)
                .disabled(disabled),
            id,
            checked,
            disabled,
        }
    }

    pub fn on_click(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.base = self
            .base
            .on_change(move |value, _, window, cx| handler(&value, window, cx));
        self
    }
}

impl RenderOnce for SettingsSwitch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let offset = spring(
            (self.id.clone(), "thumb"),
            if self.checked { px(16.) } else { px(0.) },
            cx.theme().motion_tokens().spring_move,
            window,
            cx,
        );
        let track = if self.checked {
            cx.theme().primary
        } else {
            cx.theme().switch
        };
        self.base
            .w(px(40.))
            .h(px(24.))
            .rounded(px(12.))
            .focus_visible(|style| style.shadow(vec![focus_ring(cx)]))
            .flex_shrink_0()
            .child(
                SwitchTrack::new((self.id, "track"))
                    .checked(self.checked)
                    .disabled(self.disabled)
                    .w(px(40.))
                    .h(px(24.))
                    .p(px(2.))
                    .rounded(px(12.))
                    .bg(if self.disabled {
                        track.opacity(0.5)
                    } else {
                        track
                    })
                    .child(
                        SwitchThumb::new(self.checked)
                            .size(px(20.))
                            .rounded(px(10.))
                            .left(offset)
                            .bg(cx.theme().switch_thumb),
                    ),
            )
    }
}

fn focus_ring(cx: &App) -> BoxShadow {
    BoxShadow {
        inset: false,
        color: cx.theme().ring,
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(2.),
    }
}
