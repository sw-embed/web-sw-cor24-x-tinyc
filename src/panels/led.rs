use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct LedPanelProps {
    pub state: u8,
}

#[function_component(LedPanel)]
pub fn led_panel(props: &LedPanelProps) -> Html {
    let bg = if props.state & 1 == 0 {
        "#a6e3a1"
    } else {
        "#313244"
    };
    html! {
        <div style="display:flex; align-items:center; gap:6px;">
            <span style="color:#bac2de; font-size:0.8rem;">{"LED D2"}</span>
            <div style={format!("width:14px; height:14px; border-radius:50%; \
                background:{bg}; border:1px solid #585b70;")} />
        </div>
    }
}
