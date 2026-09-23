use crate::*;

impl Workspace {
    pub(super) fn info_view(
        &self,
        fields: &[(String, String)],
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        panel_layout(cx)
            .child(
                panel_body("properties-body").child(
                    panel_card(cx).children(
                        fields
                            .iter()
                            .map(|(key, value)| panel_property(key.clone(), value.clone(), cx)),
                    ),
                ),
            )
            .into_any_element()
    }
}
