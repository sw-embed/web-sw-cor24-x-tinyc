use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SwitchPanelProps {
    pub pressed: bool,
    pub on_toggle: Callback<MouseEvent>,
}

#[function_component(SwitchPanel)]
pub fn switch_panel(props: &SwitchPanelProps) -> Html {
    let (bg, fg, label) = if props.pressed {
        ("#a6e3a1", "#1e1e2e", "ON")
    } else {
        ("#313244", "#9399b2", "OFF")
    };
    html! {
        <div style="display:flex; align-items:center; gap:6px;">
            <span style="color:#bac2de; font-size:0.8rem;">{"S2"}</span>
            <button onclick={props.on_toggle.clone()}
                style={format!("padding:2px 10px; border-radius:4px; font-size:0.8rem; \
                    cursor:pointer; border:1px solid #585b70; \
                    background:{bg}; color:{fg};")}>
                { label }
            </button>
        </div>
    }
}
