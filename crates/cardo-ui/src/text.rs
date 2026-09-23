use gpui_kit::*;

/// A single-line label whose full-text hint exists only when GPUI truncates it.
pub struct CompactText {
    element: Stateful<Div>,
    text: SharedString,
    layout: TextLayout,
}

pub fn compact_text(id: impl Into<ElementId>, text: impl Into<SharedString>) -> CompactText {
    let text = text.into();
    let label = StyledText::new(text.clone());
    let layout = label.layout().clone();
    CompactText {
        element: div()
            .id(id)
            .min_w_0()
            .truncate()
            .aria_label(text.clone())
            .child(label),
        text,
        layout,
    }
}

impl Styled for CompactText {
    fn style(&mut self) -> &mut StyleRefinement {
        self.element.style()
    }
}

impl InteractiveElement for CompactText {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.element.interactivity()
    }
}

impl StatefulInteractiveElement for CompactText {}

impl IntoElement for CompactText {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for CompactText {
    type RequestLayoutState = <Stateful<Div> as Element>::RequestLayoutState;
    type PrepaintState = <Stateful<Div> as Element>::PrepaintState;

    fn id(&self) -> Option<ElementId> {
        Element::id(&self.element)
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.element.source_location()
    }

    fn a11y_role(&self) -> Option<accesskit::Role> {
        self.element.a11y_role()
    }

    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        self.element.write_a11y_info(node);
    }

    fn a11y_synthetic_children(
        &mut self,
        prepaint: &mut Self::PrepaintState,
        builder: &mut A11ySubtreeBuilder,
    ) {
        Element::a11y_synthetic_children(&mut self.element, prepaint, builder);
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.element.request_layout(id, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        // Read the final shaped text rather than estimating by character count
        // or font size. This also covers middle ellipsis and line clamping.
        if self.layout.text() != self.text.as_ref() {
            crate::tooltip::text_tooltip(&mut self.element, self.text.clone(), bounds);
        }
        self.element
            .prepaint(id, inspector_id, bounds, state, window, cx)
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.element.paint(
            id,
            inspector_id,
            bounds,
            request_layout,
            prepaint,
            window,
            cx,
        );
    }
}

pub fn body_text(text: impl Into<SharedString>) -> Div {
    div().min_w_0().whitespace_normal().child(text.into())
}
