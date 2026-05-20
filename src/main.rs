mod compiler;
mod demos;
mod editor;
mod highlight;
mod panels;

use std::cell::RefCell;
use std::rc::Rc;

use cor24_assembler::AssembledLine;
use cor24_emulator::EmulatorCore;
use cor24_emulator::peripherals::i2c::devices::{Add1Device, Ds1307Device};
use cor24_emulator::peripherals::spi::devices::SdCardDevice;
use wasm_bindgen::JsCast;
use web_sys::{HtmlSelectElement, KeyboardEvent};
use yew::prelude::*;

use editor::Editor;
use panels::{I2cPanel, LedPanel, RegistersPanel, SwitchPanel, UartPanel};

/// Attach the default I2C/SPI device set that the interactive I2C/SPI demos
/// rely on. Devices that aren't touched by a given demo simply sit on the bus
/// unobserved — no interference with CPU/UART-only demos.
fn attach_default_peripherals(e: &mut EmulatorCore) {
    let _ = e.attach_i2c_device(Add1Device::new(0x50, 256));
    let _ = e.attach_i2c_device(Ds1307Device::new(0x68));

    // 8-sector image with recognizable patterns so the SD card demo
    // reads "00 01 02 ... 0F" from the first 16 bytes of sector 0.
    let mut sd = SdCardDevice::new();
    let mut img = Vec::with_capacity(8 * 512);
    for sector in 0u16..8 {
        for b in 0u16..256 {
            img.push((sector * 16 + b) as u8);
        }
        img.resize(((sector + 1) * 512) as usize, 0xFF);
    }
    sd.replace_image(img);
    let _ = e.attach_spi_device(sd);
}

#[function_component(App)]
fn app() -> Html {
    let source = use_state(|| demos::DEFAULT_SOURCE.to_string());

    // Compilation state
    let listing = use_state(Vec::<AssembledLine>::new);
    let compile_error = use_state(|| None::<compiler::CompileError>);

    // Emulator (mutable ref, survives re-renders)
    let emu: Rc<RefCell<EmulatorCore>> = use_mut_ref(EmulatorCore::new);

    // Emulator display state (updated each tick)
    let uart_output = use_state(String::new);
    let i2c_output = use_state(String::new);
    let registers = use_state(|| [0u32; 8]);
    let pc_val = use_state(|| 0u32);
    let cond_flag = use_state(|| false);
    let led_state = use_state(|| 0u8);
    let running = use_state(|| false);
    let halted = use_state(|| false);
    let instr_count = use_state(|| 0u64);
    let status_msg = use_state(|| String::from("Ready"));
    let runtime_error_line = use_state(|| None::<usize>);

    // Switch S2
    let switch_pressed = use_state(|| false);

    // UART input buffer (keyboard → emulator, drained in run loop)
    let uart_input: Rc<RefCell<std::collections::VecDeque<u8>>> =
        use_mut_ref(std::collections::VecDeque::new);

    // Interval handle
    let interval_handle = use_mut_ref(|| None::<gloo_timers::callback::Interval>);

    // Loading demo
    let loading = use_state(|| false);

    // --- Callbacks ---

    let on_source_change = {
        let source = source.clone();
        Callback::from(move |value: String| source.set(value))
    };

    let on_run = {
        let source = source.clone();
        let listing = listing.clone();
        let compile_error = compile_error.clone();
        let emu = emu.clone();
        let uart_input = uart_input.clone();
        let uart_output = uart_output.clone();
        let i2c_output = i2c_output.clone();
        let registers = registers.clone();
        let pc_val = pc_val.clone();
        let cond_flag = cond_flag.clone();
        let led_state = led_state.clone();
        let running = running.clone();
        let halted = halted.clone();
        let instr_count = instr_count.clone();
        let status_msg = status_msg.clone();
        let runtime_error_line = runtime_error_line.clone();
        let interval_handle = interval_handle.clone();
        let switch_pressed = switch_pressed.clone();

        Callback::from(move |_: MouseEvent| {
            // Stop any existing run loop
            *interval_handle.borrow_mut() = None;

            // Compile
            let output = compiler::compile(&source);
            listing.set(output.listing.clone());
            runtime_error_line.set(None);

            if let Some(err) = output.error {
                compile_error.set(Some(err));
                running.set(false);
                halted.set(false);
                status_msg.set("Compile error".into());
                return;
            }
            compile_error.set(None);

            // Reset emulator and load binary
            {
                let mut e = emu.borrow_mut();
                *e = EmulatorCore::new();
                attach_default_peripherals(&mut e);
                e.load_program(0, &output.bytes);
                e.load_program_extent(output.bytes.len() as u32);
                e.set_button_pressed(*switch_pressed);
                e.resume();
            }

            // Reset display state
            uart_output.set(String::new());
            i2c_output.set(String::new());
            registers.set([0u32; 8]);
            pc_val.set(0);
            cond_flag.set(false);
            led_state.set(0);
            halted.set(false);
            instr_count.set(0);
            status_msg.set("Running".into());
            running.set(true);

            // Clear input buffer
            uart_input.borrow_mut().clear();

            // Start run loop
            let emu = emu.clone();
            let uart_input = uart_input.clone();
            let uart_output = uart_output.clone();
            let i2c_output = i2c_output.clone();
            let registers = registers.clone();
            let pc_val = pc_val.clone();
            let cond_flag = cond_flag.clone();
            let led_state = led_state.clone();
            let running = running.clone();
            let halted = halted.clone();
            let instr_count = instr_count.clone();
            let status_msg = status_msg.clone();
            let runtime_error_line = runtime_error_line.clone();
            let listing = listing.clone();
            let interval_handle2 = interval_handle.clone();

            let interval = gloo_timers::callback::Interval::new(16, move || {
                let mut e = emu.borrow_mut();

                // Drain keyboard input buffer into UART RX when free
                {
                    let mut buf = uart_input.borrow_mut();
                    if !buf.is_empty()
                        && (e.read_byte(0xFF0101) & 0x01 == 0)
                        && let Some(byte) = buf.pop_front()
                    {
                        e.send_uart_byte(byte);
                    }
                }

                let batch = e.run_batch(10_000);

                // Update display state
                uart_output.set(e.get_uart_output().to_string());
                i2c_output.set(e.format_i2c_log());
                let mut regs = [0u32; 8];
                for (i, reg) in regs.iter_mut().enumerate() {
                    *reg = e.get_reg(i as u8);
                }
                registers.set(regs);
                pc_val.set(e.pc());
                cond_flag.set(e.condition_flag());
                led_state.set(e.get_led());
                instr_count.set(e.instructions_count());

                let stop = match batch.reason {
                    cor24_emulator::StopReason::Halted => {
                        halted.set(true);
                        status_msg.set("Halted".into());
                        true
                    }
                    cor24_emulator::StopReason::InvalidInstruction(op) => {
                        let pc = e.pc();
                        let line = compiler::pc_to_listing_line(&listing, pc);
                        runtime_error_line.set(line);
                        halted.set(true);
                        status_msg.set(format!("Invalid instruction: {op:#04x} at PC={pc:#06x}"));
                        true
                    }
                    cor24_emulator::StopReason::Paused => {
                        status_msg.set("Paused".into());
                        true
                    }
                    _ => false,
                };

                if stop {
                    running.set(false);
                    *interval_handle2.borrow_mut() = None;
                }
            });

            *interval_handle.borrow_mut() = Some(interval);
        })
    };

    let on_stop = {
        let emu = emu.clone();
        let interval_handle = interval_handle.clone();
        let running = running.clone();
        let status_msg = status_msg.clone();
        Callback::from(move |_: MouseEvent| {
            emu.borrow_mut().pause();
            *interval_handle.borrow_mut() = None;
            running.set(false);
            status_msg.set("Stopped".into());
        })
    };

    let on_key = {
        let uart_input = uart_input.clone();
        Callback::from(move |e: KeyboardEvent| {
            e.prevent_default();
            let key = e.key();
            let byte = if key.len() == 1 {
                key.as_bytes()[0]
            } else if key == "Enter" {
                b'\n'
            } else if key == "Backspace" {
                0x08
            } else {
                return;
            };
            uart_input.borrow_mut().push_back(byte);
        })
    };

    let on_switch_toggle = {
        let switch_pressed = switch_pressed.clone();
        let emu = emu.clone();
        Callback::from(move |_: MouseEvent| {
            let new_val = !*switch_pressed;
            switch_pressed.set(new_val);
            emu.borrow_mut().set_button_pressed(new_val);
        })
    };

    let on_demo_select = {
        let source = source.clone();
        let compile_error = compile_error.clone();
        let listing = listing.clone();
        let interval_handle = interval_handle.clone();
        let running = running.clone();
        let status_msg = status_msg.clone();
        let loading = loading.clone();
        Callback::from(move |e: Event| {
            let Some(select) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
            else {
                return;
            };
            let value = select.value();
            if value.is_empty() {
                return;
            }
            select.set_value("");

            // Stop any running emulator
            *interval_handle.borrow_mut() = None;
            running.set(false);

            // Inline demos: skip the GitHub fetch
            if let Some(src) = demos::inline_source(&value) {
                source.set(src.to_string());
                compile_error.set(None);
                listing.set(Vec::new());
                status_msg.set("Ready".into());
                return;
            }

            // Fetch from GitHub
            let url = format!("{}{value}", demos::RAW_BASE);
            let source = source.clone();
            let compile_error = compile_error.clone();
            let listing = listing.clone();
            let status_msg = status_msg.clone();
            let loading = loading.clone();
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match gloo_net::http::Request::get(&url).send().await {
                    Ok(resp) if resp.ok() => {
                        if let Ok(text) = resp.text().await {
                            source.set(text);
                            compile_error.set(None);
                            listing.set(Vec::new());
                            status_msg.set("Ready".into());
                        }
                    }
                    _ => {
                        source.set(format!("// Failed to fetch {value}"));
                    }
                }
                loading.set(false);
            });
        })
    };

    // --- Error lines for highlighting ---
    let c_error_line = compile_error
        .as_ref()
        .filter(|e| e.source == compiler::ErrorSource::C)
        .and_then(|e| e.line);
    let asm_error_line = compile_error
        .as_ref()
        .filter(|e| e.source == compiler::ErrorSource::Assembler)
        .and_then(|e| e.line)
        .or(*runtime_error_line);

    // --- Render ---
    html! {
        <main style="display:flex; flex-direction:column; height:100vh; padding:16px; gap:12px;">
            // GitHub corner
            <a href="https://github.com/sw-embed/web-sw-cor24-x-tinyc" aria-label="View source on GitHub"
                target="_blank" style="position:absolute; top:0; right:0; z-index:100;">
                <svg width="80" height="80" viewBox="0 0 250 250"
                    style="fill:#89b4fa; color:#1e1e2e;" aria-hidden="true">
                    <path d="M0,0 L115,115 L130,115 L142,142 L250,250 L250,0 Z" />
                    <path d="M128.3,109.0 C113.8,99.7 119.0,89.6 119.0,89.6 C122.0,82.7 120.5,78.6 \
                        120.5,78.6 C119.2,72.0 123.4,76.3 123.4,76.3 C127.3,80.9 125.5,87.3 125.5,87.3 \
                        C122.9,97.6 130.6,101.9 134.4,103.2" fill="currentColor"
                        style="transform-origin:130px 106px;" />
                    <path d="M115.0,115.0 C114.9,115.1 118.7,116.5 119.8,115.4 L133.7,101.6 C136.9,99.2 \
                        139.9,98.4 142.2,98.6 C133.8,88.0 127.5,74.4 143.8,58.0 C148.5,53.4 154.0,51.2 \
                        159.7,51.0 C160.3,49.4 163.2,43.6 171.4,40.1 C171.4,40.1 176.1,42.5 178.8,56.2 \
                        C183.1,58.6 187.2,61.8 190.9,65.4 C194.5,69.0 197.7,73.2 200.1,77.6 C213.8,80.2 \
                        216.3,84.9 216.3,84.9 C212.7,93.1 206.9,96.0 205.4,96.6 C205.1,102.4 203.0,107.8 \
                        198.3,112.5 C181.9,128.9 168.3,122.5 157.7,114.1 C157.9,116.9 156.7,120.9 \
                        152.7,124.9 L141.0,136.5 C139.8,137.7 141.6,141.9 141.8,141.8 Z"
                        fill="currentColor" />
                </svg>
            </a>

            <h1 style="font-size:1.4rem; color:#89b4fa;">
                {"web-sw-cor24-x-tinyc"}
                <span style="font-size:0.8rem; color:#bac2de; margin-left:8px;">
                    {"COR24 tiny C cross-compiler in your browser"}
                </span>
            </h1>

            <div style="display:flex; flex:1; gap:12px; min-height:0;">
                // C source editor
                <div style="flex:1; min-width:0; display:flex; flex-direction:column; gap:8px;">
                    <label style="font-size:0.9rem; color:#cdd6f4; font-weight:600;">{"C Source"}</label>
                    <Editor value={AttrValue::from((*source).clone())} on_change={on_source_change}
                            error_line={c_error_line} />
                </div>

                // Listing
                <div style="flex:1; min-width:0; display:flex; flex-direction:column; gap:8px;">
                    <label style="font-size:0.9rem; color:#cdd6f4; font-weight:600;">{"Listing"}</label>
                    { panels::listing::render(&listing, asm_error_line) }
                </div>

                // Emulator panel
                <div style="flex:1; min-width:0; display:flex; flex-direction:column; gap:8px;">
                    <label style="font-size:0.9rem; color:#cdd6f4; font-weight:600;">{"Emulator"}</label>
                    <div style="flex:1; display:flex; flex-direction:column; gap:8px; \
                                background:#181825; border:1px solid #313244; border-radius:6px; \
                                padding:12px; overflow:auto;">

                        // Compile error
                        if let Some(err) = compile_error.as_ref() {
                            <div style="margin-bottom:8px;">
                                <div style="color:#f38ba8; font-weight:600; font-size:0.8rem; margin-bottom:2px;">
                                    { match err.source {
                                        compiler::ErrorSource::C => "C error".to_string(),
                                        compiler::ErrorSource::Header => {
                                            format!("Header error ({})",
                                                err.header.unwrap_or("unknown"))
                                        }
                                        compiler::ErrorSource::Assembler => "Assembler error".to_string(),
                                    }}
                                    if let Some(line) = err.line {
                                        {format!(" line {line}")}
                                    }
                                </div>
                                <pre style="color:#f38ba8; margin:0; white-space:pre-wrap; font-size:0.8rem;">
                                    {&err.message}
                                </pre>
                            </div>
                        }

                        <UartPanel
                            output={AttrValue::from((*uart_output).clone())}
                            running={*running}
                            halted={*halted}
                            on_key={on_key}
                        />

                        <I2cPanel
                            output={AttrValue::from((*i2c_output).clone())}
                            running={*running}
                            halted={*halted}
                        />

                        <RegistersPanel regs={*registers} pc={*pc_val} cond={*cond_flag} />

                        // Hardware I/O: LED + Switch
                        <div style="display:flex; gap:16px; align-items:center;">
                            <LedPanel state={*led_state} />
                            <SwitchPanel pressed={*switch_pressed} on_toggle={on_switch_toggle} />
                        </div>

                        // Status bar
                        <div style="display:flex; justify-content:space-between; align-items:center; \
                                    font-size:0.8rem; color:#bac2de; border-top:1px solid #313244; \
                                    padding-top:6px;">
                            <span>{&*status_msg}</span>
                            <span>{format!("{} instructions", *instr_count)}</span>
                        </div>
                    </div>
                </div>
            </div>

            // Button bar
            <div style="display:flex; gap:12px; align-items:center;">
                <button onclick={on_run}
                    style="padding:8px 24px; background:#89b4fa; color:#1e1e2e; \
                           border:none; border-radius:6px; font-size:1rem; font-weight:600; cursor:pointer;">
                    {"Compile & Run"}
                </button>

                if *running {
                    <button onclick={on_stop}
                        style="padding:8px 24px; background:#f38ba8; color:#1e1e2e; \
                               border:none; border-radius:6px; font-size:1rem; font-weight:600; cursor:pointer;">
                        {"Stop"}
                    </button>
                }

                <select onchange={on_demo_select}
                    style="padding:6px 12px; background:#313244; color:#cdd6f4; border:1px solid #585b70; \
                           border-radius:6px; font-size:0.85rem; cursor:pointer;">
                    <option value="" selected=true disabled=true>
                        { if *loading { "Loading..." } else { "Load demo..." } }
                    </option>
                    <optgroup label="Interactive">
                        { for demos::INTERACTIVE_DEMOS.iter().map(|(id, label, _)| html! {
                            <option value={*id}>{*label}</option>
                        }) }
                    </optgroup>
                    <optgroup label="tc24r demos">
                        { for demos::DEMOS.iter().map(|(file, label)| html! {
                            <option value={*file}>{format!("{file} — {label}")}</option>
                        }) }
                    </optgroup>
                </select>
            </div>

            // Bundled headers (collapsible)
            <details style="font-size:0.9rem;">
                <summary style="color:#bac2de; cursor:pointer; user-select:none;">
                    {"Bundled headers (stdio.h, stdlib.h, string.h, cor24.h, stdbool.h)"}
                </summary>
                <div style="display:flex; gap:8px; margin-top:8px; max-height:300px; overflow:auto;">
                    { for compiler::HEADERS.iter().map(|(name, content)| html! {
                        <details style="flex:1; min-width:0;">
                            <summary style="color:#89b4fa; cursor:pointer; font-family:monospace; \
                                            font-size:0.85rem; padding:4px 8px; background:#181825; \
                                            border-radius:4px 4px 0 0; border:1px solid #313244;">
                                {*name}
                            </summary>
                            <pre style="margin:0; padding:8px; background:#11111b; color:#cdd6f4; \
                                        border:1px solid #313244; border-top:none; border-radius:0 0 4px 4px; \
                                        font-size:12px; line-height:1.4; white-space:pre-wrap; \
                                        max-height:250px; overflow:auto;">
                                {*content}
                            </pre>
                        </details>
                    }) }
                </div>
            </details>

            // Footer
            <div style="display:flex; gap:8px; align-items:center; flex-wrap:wrap; \
                        font-size:0.9rem; color:#bac2de; padding-top:4px;">
                <span>{"\u{00a9} 2026 Michael A. Wright"}</span>
                <span>{"\u{00b7}"}</span>
                <span>{"MIT License"}</span>
                <span>{"\u{00b7}"}</span>
                <a href="https://makerlisp.com" target="_blank"
                    style="color:#89b4fa; text-decoration:none;">{"COR24-TB"}</a>
                <span>{"\u{00b7}"}</span>
                <a href="https://software-wrighter-lab.github.io/" target="_blank"
                    style="color:#89b4fa; text-decoration:none;">{"Blog"}</a>
                <span>{"\u{00b7}"}</span>
                <a href="https://discord.com/invite/Ctzk5uHggZ" target="_blank"
                    style="color:#89b4fa; text-decoration:none;">{"Discord"}</a>
                <span>{"\u{00b7}"}</span>
                <a href="https://www.youtube.com/@SoftwareWrighter" target="_blank"
                    style="color:#89b4fa; text-decoration:none;">{"YouTube"}</a>
                <span>{"\u{00b7}"}</span>
                <span>{ format!("{} \u{00b7} {} \u{00b7} {}",
                    env!("BUILD_HOST"),
                    env!("BUILD_SHA"),
                    env!("BUILD_TIMESTAMP"),
                ) }</span>
            </div>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
