//! Assembly listing renderer.
//!
//! Kept as a free function rather than a `#[function_component]` because
//! `AssembledLine` (defined upstream in `cor24-emulator`) doesn't impl
//! `PartialEq`, which Yew's `Properties` derive requires.

use cor24_emulator::AssembledLine;
use yew::prelude::*;

pub fn render(listing: &[AssembledLine], error_line: Option<usize>) -> Html {
    if listing.is_empty() {
        return html! {
            <pre style="flex:1; background:#181825; color:#f9e2af; border:1px solid #313244; \
                        border-radius:6px; padding:12px; font-family:monospace; font-size:14px; \
                        overflow:auto; white-space:pre;" />
        };
    }

    let width = listing.len().to_string().len();

    html! {
        <div style="flex:1; display:flex; background:#181825; border:1px solid #313244; \
                    border-radius:6px; overflow:auto; font-family:monospace; font-size:13px; \
                    line-height:1.5;">
            <pre style="margin:0; padding:12px 8px 12px 0; text-align:right; color:#bac2de; \
                        user-select:none; background:#11111b; border-right:1px solid #313244; \
                        white-space:pre;">
                { for listing.iter().enumerate().map(|(i, _)| {
                    let n = i + 1;
                    let is_err = error_line == Some(n);
                    let style = if is_err { "color:#f38ba8; background:rgba(243,139,168,0.15);" } else { "" };
                    html! { <span {style}>{format!("{:>width$}\n", n)}</span> }
                }) }
            </pre>
            <pre style="margin:0; padding:12px; white-space:pre; flex:1;">
                { for listing.iter().enumerate().map(|(i, line)| {
                    let n = i + 1;
                    let is_err = error_line == Some(n);
                    let bg = if is_err { "background:rgba(243,139,168,0.15);" } else { "" };
                    let formatted = format_listing_line(line);
                    if line.bytes.is_empty() {
                        html! { <div style={format!("color:#f9e2af;{bg}")}>{formatted}</div> }
                    } else {
                        let addr_end = 6;
                        let hex_start = 8;
                        let hex_end = hex_start + 14;
                        html! {
                            <div style={bg.to_string()}>
                                <span style="color:#bac2de;">{formatted[..addr_end].to_string()}</span>
                                <span style="color:#bac2de;">{formatted[addr_end..hex_start].to_string()}</span>
                                <span style="color:#a6e3a1;">{formatted[hex_start..hex_end.min(formatted.len())].to_string()}</span>
                                if formatted.len() > hex_end {
                                    <span style="color:#f9e2af;">{formatted[hex_end..].to_string()}</span>
                                }
                            </div>
                        }
                    }
                }) }
            </pre>
        </div>
    }
}

fn format_listing_line(line: &AssembledLine) -> String {
    if line.bytes.is_empty() {
        format!("{:>22}{}", "", line.source)
    } else {
        let hex: String = line.bytes.iter().map(|b| format!("{b:02x} ")).collect();
        format!(
            "{:06x}  {:<14}{}",
            line.address,
            hex.trim_end(),
            line.source
        )
    }
}
