use web_sys::KeyboardEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct UartPanelProps {
    pub output: AttrValue,
    pub running: bool,
    pub halted: bool,
    pub on_key: Callback<KeyboardEvent>,
}

#[function_component(UartPanel)]
pub fn uart_panel(props: &UartPanelProps) -> Html {
    html! {
        <div style="flex:1; min-height:80px;">
            <div style="color:#bac2de; font-size:0.8rem; margin-bottom:2px;">
                {"UART"}
                if props.running {
                    <span style="color:#a6adc8;">{" (type here for input)"}</span>
                }
            </div>
            <div onkeydown={props.on_key.clone()} tabindex="0"
                style="background:#11111b; color:#a6e3a1; padding:8px; border-radius:4px; \
                       font-family:monospace; font-size:13px; white-space:pre-wrap; \
                       min-height:40px; max-height:200px; overflow:auto; \
                       outline:none; cursor:text; \
                       border:1px solid transparent;">
                { if props.output.is_empty() && !props.running && !props.halted {
                    html! { <span style="color:#a6adc8;">{"(no output)"}</span> }
                } else {
                    html! { {props.output.clone()} }
                }}
            </div>
        </div>
    }
}
