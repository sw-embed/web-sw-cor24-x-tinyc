//! Demo C-source catalogues used by the Load-demo dropdown.
//!
//! Two flavors:
//! - `INTERACTIVE_DEMOS`: hand-written demos kept inline so they don't have to
//!   be fetched at runtime (used by the I/O playground panels).
//! - `DEMO10B_SRC` + `DEMOS`: the language-feature demos that live in
//!   `sw-cor24-x-tinyc/demos/` and are fetched from GitHub on demand. `demo10b`
//!   is inline because it's a web-only variant of CLI demo10.

pub const DEFAULT_SOURCE: &str = r#"// Hello, World! — COR24 C with printf and LED
#include <stdio.h>

int main() {
    printf("Hello, World!\n");
    printf("2 + 2 = %d\n", 2 + 2);

    // Light LED D2 (active-low: write 0 to turn on)
    *(char *)0xFF0000 = 0;

    return 42;
}
"#;

/// Built-in interactive demos (inline source, not fetched from GitHub).
pub const INTERACTIVE_DEMOS: &[(&str, &str, &str)] = &[
    (
        "hello",
        "Hello, World! (printf + LED)",
        r#"// Hello, World! — COR24 C with printf and LED
#include <stdio.h>

int main() {
    printf("Hello, World!\n");
    printf("2 + 2 = %d\n", 2 + 2);

    // Light LED D2 (active-low: write 0 to turn on)
    *(char *)0xFF0000 = 0;

    return 42;
}
"#,
    ),
    (
        "echo",
        "UART echo (type to see characters)",
        r#"// UART echo — type in the terminal, characters echo back
// Demonstrates: interrupt-driven UART RX, polling UART TX
// Uses __attribute__((interrupt)) for the ISR

#define UART_DATA   0xFF0100
#define UART_STATUS 0xFF0101
#define INT_ENABLE  0xFF0010

void putc(int c) {
    while (*(char *)UART_STATUS & 0x80) {}
    *(char *)UART_DATA = c;
}

// ISR: called on each UART RX byte
__attribute__((interrupt))
void uart_isr() {
    int c = *(char *)UART_DATA;  // read & acknowledge
    putc(c);                      // echo back
    if (c == 13 || c == 10) {
        putc(62);  // '>'
        putc(32);  // ' '
    }
}

int main() {
    // Set interrupt vector
    asm("la r0,_uart_isr\nmov iv,r0");
    // Enable UART RX interrupt
    *(char *)INT_ENABLE = 1;

    putc(62); // '>'
    putc(32); // ' '

    // Spin forever (ISR handles input)
    while (1) {}
}
"#,
    ),
    (
        "led-switch",
        "LED follows switch S2",
        r#"// LED follows switch — press S2 to light LED D2
// Demonstrates: polling switch input, controlling LED output
// Click the S2 button below to toggle!

#define LED_REG  0xFF0000

int main() {
    while (1) {
        int sw = *(char *)LED_REG;
        // Switch is bit 0: 1=released, 0=pressed
        // LED is active-low: write 0=on, 1=off
        // So just write the switch state to LED — pressed=0=LED on
        *(char *)LED_REG = sw & 1;
    }
}
"#,
    ),
    (
        "counter",
        "Live counter on UART",
        r#"// Live counter — prints incrementing numbers
// Demonstrates: busy-wait loop, UART output

#include <stdio.h>

void delay() {
    int i = 0;
    while (i < 5000) { i++; }
}

int main() {
    int n = 0;
    while (1) {
        printf("%d\n", n);
        n++;
        delay();
    }
}
"#,
    ),
    (
        "adder",
        "Interactive adder (type two numbers)",
        r#"// Interactive adder — type two numbers separated by Enter
// Demonstrates: UART input parsing, printf output

#include <stdio.h>

int getc_poll() {
    while (!(*(char *)0xFF0101 & 0x01)) {}
    return *(char *)0xFF0100;
}

int read_int() {
    int n = 0;
    int started = 0;
    while (1) {
        int c = getc_poll();
        putchar(c);  // echo
        if (c >= 48 && c <= 57) {
            n = n * 10 + (c - 48);
            started = 1;
        } else if (started) {
            return n;
        }
    }
}

int main() {
    while (1) {
        printf("a? ");
        int a = read_int();
        printf("b? ");
        int b = read_int();
        printf("= %d\n", a + b);
    }
}
"#,
    ),
];

/// Web-only variant of demo10 (CLI demo10 uses a local header file demo10_io.h
/// that isn't available in the web compiler; this version exercises the same
/// #include and #pragma once features using the five bundled headers instead).
pub const DEMO10B_SRC: &str = r#"// tc24r demo10b — #include with bundled headers (web variant)
//
// The CLI demo10 tests #include with a local header (demo10_io.h).
// This web variant tests the same #include and #pragma once features
// using the five headers bundled in the web compiler instead.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <cor24.h>
#include <stdbool.h>

int main() {
    // stdio.h: printf
    printf("Include test\n");

    // stdlib.h: malloc/free
    int *p = (int *)malloc(sizeof(int));
    *p = 10;
    int v = *p;
    free(p);

    // string.h: strlen
    char *s = "hello";
    int len = strlen(s);

    // cor24.h: UART_STATUS register
    int status = UART_STATUS;

    // stdbool.h: true/false
    bool ok = true;

    if (ok && v == 10 && len == 5 && status == 0xFF0101) {
        printf("All headers OK\n");
        return 42;
    }
    return 0;
}
"#;

pub const DEMOS: &[(&str, &str)] = &[
    ("demo.c", "counter"),
    ("demo2.c", "char, pointers, casts, MMIO"),
    ("demo3.c", "hex literals, pointer arithmetic, strings"),
    ("demo4.c", "software divide and modulo"),
    ("demo5.c", "arrays"),
    ("demo6.c", "global char, pointer, array patterns"),
    ("demo7.c", "pointer subtraction"),
    ("demo8.c", "preprocessor #define"),
    ("demo9.c", "UART RX interrupt"),
    ("demo10b.c", "#include all bundled headers"),
    ("demo11.c", "logical AND/OR short-circuit"),
    ("demo12.c", "do...while loop"),
    ("demo13.c", "break and continue"),
    ("demo14.c", "increment and decrement"),
    ("demo15.c", "ternary operator"),
    ("demo16.c", "character literals"),
    ("demo17.c", "multi-declaration"),
    ("demo18.c", "sizeof operator"),
    ("demo19.c", "static and extern"),
    ("demo20.c", "statement expressions (GCC ext)"),
    ("demo21.c", "compound assignment operators"),
    ("demo22.c", "braceless control flow"),
    ("demo23.c", "enum"),
    ("demo24.c", "typedef"),
    ("demo25.c", "struct"),
    ("demo26.c", "switch/case"),
    ("demo27.c", "function prototypes"),
    ("demo28.c", "union"),
    ("demo29.c", "sizeof with array types"),
    ("demo30.c", "line continuation"),
    ("demo31.c", "tentative definitions"),
    ("demo32.c", "multi-declarator typedef"),
    ("demo33.c", "comma-separated struct/union members"),
    ("demo34.c", "multi-dimensional arrays"),
    ("demo35.c", "struct array members"),
    ("demo36.c", "forward-declared struct tags"),
    ("demo37.c", "anonymous struct/union members"),
    ("demo38.c", "struct brace initializer"),
    ("demo39.c", "printf and long branches"),
    ("demo40.c", "malloc/free (stdlib.h)"),
    ("demo41.c", "getc, atoi, string.h"),
    ("demo42.c", "nested struct (linked list)"),
    ("demo43.c", "Lisp-style cons cells"),
    ("demo44.c", "Lisp data types and printer"),
    ("demo45.c", "Lisp eval: (+ 40 2) => 42"),
    ("demo46.c", "unsigned int, shifts, comparisons"),
    ("demo47.c", "struct pointer array indexing (BUG-010)"),
    ("demo48.c", "global struct array (BUG-011)"),
    ("demo49.c", "parenthesized ptr arithmetic + arrow (BUG-012)"),
    ("demo50.c", "large local array + nested calls (BUG-013)"),
    ("demo51.c", "function pointer: basic variable call"),
    ("demo52.c", "function pointer: array dispatch table"),
    ("demo53.c", "function pointer: passed as parameter"),
    ("demo54.c", "global function pointer declaration"),
    ("demo55.c", "constant expression in array size"),
];

pub const RAW_BASE: &str = "https://raw.githubusercontent.com/sw-embed/sw-cor24-x-tinyc/main/demos/";

/// Look up an inline demo source by id (returns `None` for non-inline demos
/// that need to be fetched from GitHub).
pub fn inline_source(id: &str) -> Option<&'static str> {
    INTERACTIVE_DEMOS
        .iter()
        .find(|(demo_id, _, _)| *demo_id == id)
        .map(|(_, _, src)| *src)
        .or(if id == "demo10b.c" {
            Some(DEMO10B_SRC)
        } else {
            None
        })
}
