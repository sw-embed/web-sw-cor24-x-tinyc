//! DS1307 RTC interactive panel: time setter + battery (localStorage-persisted).
//!
//! The "battery" semantics mirror real hardware: when on, the simulated RTC
//! advances with wall-clock time and survives page reloads / Run boundaries
//! via a localStorage anchor. When off, the RTC resets to 00:00:00 at each
//! Run start (no persistence). The user can still Set time mid-Run regardless
//! of battery state — battery only controls cross-Run persistence.

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct RtcPanelProps {
    /// Currently-displayed RTC time as (hours, minutes, seconds).
    pub time: (u8, u8, u8),
    /// Whether the battery toggle is on.
    pub battery: bool,
    /// User submitted new H/M/S via the Set button.
    pub on_set: Callback<(u8, u8, u8)>,
    /// User toggled the battery checkbox.
    pub on_battery_toggle: Callback<bool>,
}

#[function_component(RtcPanel)]
pub fn rtc_panel(props: &RtcPanelProps) -> Html {
    let h_input = use_node_ref();
    let m_input = use_node_ref();
    let s_input = use_node_ref();

    let on_set = {
        let h_input = h_input.clone();
        let m_input = m_input.clone();
        let s_input = s_input.clone();
        let cb = props.on_set.clone();
        Callback::from(move |_: MouseEvent| {
            let h = read_u8(&h_input, 23);
            let m = read_u8(&m_input, 59);
            let s = read_u8(&s_input, 59);
            cb.emit((h, m, s));
        })
    };

    let on_battery_change = {
        let cb = props.on_battery_toggle.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            cb.emit(input.checked());
        })
    };

    let (h, m, s) = props.time;

    html! {
        <div style="flex:none;">
            <div style="color:#bac2de; font-size:0.8rem; margin-bottom:2px;">
                {"RTC (DS1307 @ 0x68)"}
            </div>
            <div style="display:flex; align-items:center; gap:12px; \
                       background:#11111b; padding:8px; border-radius:4px; \
                       font-family:monospace; font-size:14px;">
                <span style="color:#f9e2af; font-size:18px; min-width:90px;">
                    {format!("{h:02}:{m:02}:{s:02}")}
                </span>
                <span style="color:#a6adc8;">{"Set:"}</span>
                <input ref={h_input} type="number" min="0" max="23"
                       placeholder={format!("{h:02}")}
                       style="width:46px; padding:2px 4px; background:#181825; \
                              color:#cdd6f4; border:1px solid #313244; border-radius:3px; \
                              font-family:monospace;" />
                <span style="color:#bac2de;">{":"}</span>
                <input ref={m_input} type="number" min="0" max="59"
                       placeholder={format!("{m:02}")}
                       style="width:46px; padding:2px 4px; background:#181825; \
                              color:#cdd6f4; border:1px solid #313244; border-radius:3px; \
                              font-family:monospace;" />
                <span style="color:#bac2de;">{":"}</span>
                <input ref={s_input} type="number" min="0" max="59"
                       placeholder={format!("{s:02}")}
                       style="width:46px; padding:2px 4px; background:#181825; \
                              color:#cdd6f4; border:1px solid #313244; border-radius:3px; \
                              font-family:monospace;" />
                <button onclick={on_set}
                        style="padding:3px 10px; background:#89b4fa; color:#1e1e2e; \
                               border:none; border-radius:3px; font-weight:600; cursor:pointer;">
                    {"Set"}
                </button>
                <label style="display:flex; align-items:center; gap:4px; color:#cdd6f4; \
                              cursor:pointer; margin-left:auto;">
                    <input type="checkbox" checked={props.battery}
                           onchange={on_battery_change} />
                    {"Battery"}
                </label>
            </div>
        </div>
    }
}

fn read_u8(node: &NodeRef, max: u8) -> u8 {
    node.cast::<HtmlInputElement>()
        .and_then(|el| el.value().parse::<u32>().ok())
        .map(|v| v.min(max as u32) as u8)
        .unwrap_or(0)
}
