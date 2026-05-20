use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct I2cPanelProps {
    pub output: AttrValue,
    pub running: bool,
    pub halted: bool,
}

#[function_component(I2cPanel)]
pub fn i2c_panel(props: &I2cPanelProps) -> Html {
    html! {
        <div style="flex:1; min-height:80px;">
            <div style="color:#bac2de; font-size:0.8rem; margin-bottom:2px;">
                {"I2C bus"}
            </div>
            <div style="background:#11111b; color:#94e2d5; padding:8px; border-radius:4px; \
                       font-family:monospace; font-size:13px; white-space:pre-wrap; \
                       min-height:40px; max-height:200px; overflow:auto;">
                { if props.output.is_empty() && !props.running && !props.halted {
                    html! { <span style="color:#a6adc8;">{"(no I2C activity)"}</span> }
                } else {
                    html! { {props.output.clone()} }
                }}
            </div>
        </div>
    }
}
