//! SPI SD Card panel: file upload to swap the simulated card image,
//! IndexedDB-persisted across reloads. The default card image (an
//! 8-sector test pattern used by the `spi-sdcard` C demo) is restored
//! when the user clicks Reset.

use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SdCardPanelProps {
    /// Current image size in bytes.
    pub size: usize,
    /// True if the current image is the user-uploaded one (vs. the default).
    pub user_uploaded: bool,
    /// Optional filename of the most recent upload (display only).
    pub filename: Option<AttrValue>,
    /// User picked a file to upload.
    pub on_upload: Callback<web_sys::File>,
    /// User clicked Reset (clear upload, revert to default).
    pub on_reset: Callback<()>,
}

#[function_component(SdCardPanel)]
pub fn sdcard_panel(props: &SdCardPanelProps) -> Html {
    let file_input = use_node_ref();

    let on_change = {
        let cb = props.on_upload.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = match e.target().and_then(|t| t.dyn_into().ok()) {
                Some(i) => i,
                None => return,
            };
            if let Some(files) = input.files()
                && let Some(file) = files.get(0)
            {
                cb.emit(file);
            }
            input.set_value(""); // allow re-uploading the same file
        })
    };

    let on_reset = {
        let cb = props.on_reset.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let size_label = format_size(props.size);
    let source_label = if props.user_uploaded {
        props.filename.as_ref().map_or_else(
            || "uploaded".to_string(),
            |f| f.to_string(),
        )
    } else {
        "default test image".to_string()
    };

    html! {
        <div style="flex:none;">
            <div style="color:#bac2de; font-size:0.8rem; margin-bottom:2px;">
                {"SPI SD Card"}
            </div>
            <div style="display:flex; align-items:center; gap:12px; flex-wrap:wrap; \
                       background:#11111b; padding:8px; border-radius:4px; \
                       font-family:monospace; font-size:13px;">
                <span style="color:#fab387;">{format!("{size_label} — {source_label}")}</span>
                <label style="display:inline-flex; align-items:center; gap:4px; cursor:pointer; \
                              padding:3px 10px; background:#89b4fa; color:#1e1e2e; \
                              border-radius:3px; font-weight:600;">
                    {"Upload image"}
                    <input ref={file_input} type="file"
                           accept=".img,.iso,.bin,*/*"
                           onchange={on_change}
                           style="display:none;" />
                </label>
                <button onclick={on_reset}
                        style="padding:3px 10px; background:#313244; color:#cdd6f4; \
                               border:none; border-radius:3px; cursor:pointer;">
                    {"Reset to default"}
                </button>
            </div>
        </div>
    }
}

fn format_size(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
