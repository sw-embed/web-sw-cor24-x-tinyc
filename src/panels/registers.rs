use yew::prelude::*;

const REG_NAMES: [&str; 8] = ["r0", "r1", "r2", "fp", "sp", "z", "iv", "ir"];

#[derive(Properties, PartialEq)]
pub struct RegistersPanelProps {
    pub regs: [u32; 8],
    pub pc: u32,
    pub cond: bool,
}

#[function_component(RegistersPanel)]
pub fn registers_panel(props: &RegistersPanelProps) -> Html {
    html! {
        <div>
            <div style="color:#bac2de; font-size:0.8rem; margin-bottom:4px;">{"Registers"}</div>
            <div style="display:grid; grid-template-columns:repeat(3,1fr); gap:4px; \
                        font-family:monospace; font-size:12px;">
                { for (0..8).map(|i| {
                    html! {
                        <div style="background:#11111b; padding:2px 6px; border-radius:3px; \
                                    display:flex; justify-content:space-between;">
                            <span style="color:#bac2de;">{REG_NAMES[i]}</span>
                            <span style="color:#89b4fa;">{format!("{:06x}", props.regs[i])}</span>
                        </div>
                    }
                }) }
                <div style="background:#11111b; padding:2px 6px; border-radius:3px; \
                            display:flex; justify-content:space-between;">
                    <span style="color:#bac2de;">{"pc"}</span>
                    <span style="color:#cba6f7;">{format!("{:06x}", props.pc)}</span>
                </div>
                <div style="background:#11111b; padding:2px 6px; border-radius:3px; \
                            display:flex; justify-content:space-between;">
                    <span style="color:#bac2de;">{"c"}</span>
                    <span style="color:#f9e2af;">{ if props.cond { "1" } else { "0" } }</span>
                </div>
            </div>
        </div>
    }
}
