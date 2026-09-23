use super::Menu;
use gpui_kit::{component::button::Button, *};
use std::{cell::Cell, rc::Rc};

/// Observes the complete button layout without adding a container or changing its size.
pub struct MenuAnchor {
    button: AnyElement,
    bounds: Rc<Cell<Bounds<Pixels>>>,
}

impl IntoElement for MenuAnchor {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for MenuAnchor {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.button.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.bounds.set(bounds);
        self.button.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.button.paint(window, cx);
    }
}

pub trait MenuTrigger {
    fn measure_anchor(self, bounds: Rc<Cell<Bounds<Pixels>>>) -> MenuAnchor;
    fn popup_menu(self, build: impl Fn(&mut Window, &mut App) -> Menu + 'static) -> MenuAnchor;
    fn choice_menu(self, build: impl Fn(&mut Window, &mut App) -> Menu + 'static) -> MenuAnchor;
}

impl MenuTrigger for Button {
    fn measure_anchor(self, bounds: Rc<Cell<Bounds<Pixels>>>) -> MenuAnchor {
        MenuAnchor {
            button: self.into_any_element(),
            bounds,
        }
    }

    fn choice_menu(self, build: impl Fn(&mut Window, &mut App) -> Menu + 'static) -> MenuAnchor {
        let bounds = Rc::new(Cell::new(Bounds::<Pixels>::default()));
        let measured = bounds.clone();
        self.on_click(move |_, window, cx| {
            let anchor = bounds.get();
            let mut menu = build(window, cx);
            menu.width = Some((anchor.size.width + px(40.)).max(px(240.)));
            menu.align_end = true;
            menu.show_at(anchor, window, cx);
        })
        .measure_anchor(measured)
    }

    fn popup_menu(self, build: impl Fn(&mut Window, &mut App) -> Menu + 'static) -> MenuAnchor {
        let bounds = Rc::new(Cell::new(Bounds::<Pixels>::default()));
        let measured = bounds.clone();
        self.on_click(move |_, window, cx| {
            let anchor = bounds.get();
            build(window, cx).show_native(point(anchor.left(), anchor.bottom()), window, cx)
        })
        .measure_anchor(measured)
    }
}
