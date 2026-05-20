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
    (
        "i2c-add1",
        "I2C: Add1 ping (write 0x42, read 0x43)",
        r#"// I2C Add1 Ping — bit-bang the bus to talk to the emulator's
// `add1` test slave at 0x50. Writes 0x42, reads it back; add1
// returns byte+1, so the read yields 0x43. Watch the I2C panel
// for START / ADDR W / WR / ADDR R / RD / STOP events.
//
// The web UI auto-attaches add1@0x50 on every Run.

#define I2C_SCL     ((char *)0xFF0020)
#define I2C_SDA     ((char *)0xFF0021)
#define UART_DATA   ((char *)0xFF0100)
#define UART_STATUS ((char *)0xFF0101)

void putc(int c) {
    while (*UART_STATUS & 0x80) {}
    *UART_DATA = c;
}

void print_hex(int byte) {
    int hi = (byte >> 4) & 0x0F;
    int lo = byte & 0x0F;
    putc(hi < 10 ? '0' + hi : 'A' + hi - 10);
    putc(lo < 10 ? '0' + lo : 'A' + lo - 10);
}

// --- I2C bit-bang primitives (no clock delays; emulator is atomic) ---

void i2c_start(void) {
    *I2C_SDA = 1;
    *I2C_SCL = 1;
    *I2C_SDA = 0;    // SDA falls while SCL high = START
    *I2C_SCL = 0;
}

void i2c_stop(void) {
    *I2C_SDA = 0;
    *I2C_SCL = 1;
    *I2C_SDA = 1;    // SDA rises while SCL high = STOP
    *I2C_SCL = 0;
}

int i2c_write(int byte) {
    int i;
    int ack;
    for (i = 7; i >= 0; i = i - 1) {
        *I2C_SDA = (byte >> i) & 1;
        *I2C_SCL = 1;
        *I2C_SCL = 0;
    }
    *I2C_SDA = 1;    // release for slave
    *I2C_SCL = 1;
    ack = *I2C_SDA & 1;
    *I2C_SCL = 0;
    return ack;
}

int i2c_read(void) {
    int i;
    int byte = 0;
    for (i = 0; i < 8; i = i + 1) {
        *I2C_SDA = 1;
        *I2C_SCL = 1;
        byte = (byte << 1) | (*I2C_SDA & 1);
        *I2C_SCL = 0;
    }
    *I2C_SDA = 0;    // master ACK
    *I2C_SCL = 1;
    *I2C_SCL = 0;
    return byte;
}

int main(void) {
    int b;

    // Write 0x42 to add1@0x50
    i2c_start();
    i2c_write(0xA0);    // 0x50 << 1 | W
    i2c_write(0x42);
    i2c_stop();

    // Restart, read back — add1 returns byte+1 = 0x43
    i2c_start();
    i2c_write(0xA1);    // 0x50 << 1 | R
    b = i2c_read();
    i2c_stop();

    print_hex(b);
    putc('\n');

    return 0;
}
"#,
    ),
    (
        "i2c-ds1307",
        "I2C: DS1307 RTC read (HH:MM:SS)",
        r#"// I2C DS1307 Read — read seconds/minutes/hours from the RTC at
// I2C addr 0x68, print as "HH:MM:SS\n". The web UI auto-attaches
// ds1307@0x68 on every Run; default time is 00:00:00 so the output
// is "00:00:00\n". Watch the I2C panel for the full transaction.

#define I2C_SCL     ((char *)0xFF0020)
#define I2C_SDA     ((char *)0xFF0021)
#define UART_DATA   ((char *)0xFF0100)
#define UART_STATUS ((char *)0xFF0101)

void putc(int c) {
    while (*UART_STATUS & 0x80) {}
    *UART_DATA = c;
}

// Print a BCD byte as two ASCII decimal digits.
void print_bcd(int b) {
    putc('0' + ((b >> 4) & 0x0F));
    putc('0' + (b & 0x0F));
}

void i2c_start(void) {
    *I2C_SDA = 1; *I2C_SCL = 1; *I2C_SDA = 0; *I2C_SCL = 0;
}

void i2c_stop(void) {
    *I2C_SDA = 0; *I2C_SCL = 1; *I2C_SDA = 1; *I2C_SCL = 0;
}

int i2c_write(int byte) {
    int i;
    int ack;
    for (i = 7; i >= 0; i = i - 1) {
        *I2C_SDA = (byte >> i) & 1;
        *I2C_SCL = 1;
        *I2C_SCL = 0;
    }
    *I2C_SDA = 1;
    *I2C_SCL = 1;
    ack = *I2C_SDA & 1;
    *I2C_SCL = 0;
    return ack;
}

int i2c_read(void) {
    int i;
    int byte = 0;
    for (i = 0; i < 8; i = i + 1) {
        *I2C_SDA = 1;
        *I2C_SCL = 1;
        byte = (byte << 1) | (*I2C_SDA & 1);
        *I2C_SCL = 0;
    }
    *I2C_SDA = 0;
    *I2C_SCL = 1;
    *I2C_SCL = 0;
    return byte;
}

int main(void) {
    int s, m, h;

    // Set DS1307 register pointer = 0 (Seconds)
    i2c_start();
    i2c_write(0xD0);    // 0x68 << 1 | W
    i2c_write(0x00);

    // Restart, read S, M, H (pointer auto-increments)
    i2c_start();
    i2c_write(0xD1);    // 0x68 << 1 | R
    s = i2c_read();
    m = i2c_read();
    h = i2c_read();
    i2c_stop();

    print_bcd(h & 0x3F);    // mask 12/24-hour mode bit
    putc(':');
    print_bcd(m);
    putc(':');
    print_bcd(s & 0x7F);    // mask Clock Halt (CH) bit
    putc('\n');

    return 0;
}
"#,
    ),
    (
        "spi-sdcard",
        "SPI: SD Card read sector 0 (first 16 bytes)",
        r#"// SPI SD Card Read — run the SD-SPI init handshake, read sector 0,
// print the first 16 bytes as hex pairs to UART.
//
// The web UI auto-attaches sdcard with an 8-sector test image whose
// sector 0 contains 0x00,0x01,...,0xFF — so the expected output is:
//     00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F
//
// The I2C panel stays empty (this is a SPI demo).
//
// MMIO:
//   0xFF0030 SPI_DATA — write = MOSI bit; read = last MISO bit
//   0xFF0031 SPI_SCLK — bit 0 drives SCLK
//   0xFF0032 SPI_SELN — bit 0 drives CS (active low; 1 = idle)

#define SPI_DATA ((char *)0xFF0030)
#define SPI_SCLK ((char *)0xFF0031)
#define SPI_SELN ((char *)0xFF0032)
#define UART_DATA   ((char *)0xFF0100)
#define UART_STATUS ((char *)0xFF0101)

void putc(int c) {
    while (*UART_STATUS & 0x80) {}
    *UART_DATA = c;
}

void print_hex(int byte) {
    int hi = (byte >> 4) & 0x0F;
    int lo = byte & 0x0F;
    putc(hi < 10 ? '0' + hi : 'A' + hi - 10);
    putc(lo < 10 ? '0' + lo : 'A' + lo - 10);
}

void cs_low(void)  { *SPI_SELN = 0; }
void cs_high(void) { *SPI_SELN = 1; }

// Exchange one byte: drive 8 MOSI bits MSB-first, sample 8 MISO bits.
int spi_xchg(int byte) {
    int i;
    int acc = 0;
    for (i = 7; i >= 0; i = i - 1) {
        *SPI_DATA = (byte >> i) & 1;
        *SPI_SCLK = 1;
        acc = (acc << 1) | (*SPI_DATA & 1);
        *SPI_SCLK = 0;
    }
    return acc;
}

// Send 6-byte SD command: cmd, arg[0..3], crc.
void sd_cmd(int cmd, int a0, int a1, int a2, int a3, int crc) {
    spi_xchg(cmd);
    spi_xchg(a0);
    spi_xchg(a1);
    spi_xchg(a2);
    spi_xchg(a3);
    spi_xchg(crc);
}

// Clock 0xFF until response with bit 7 clear (R1). Bounded to 256 retries.
int sd_r1(void) {
    int i;
    int r;
    for (i = 0; i < 256; i = i + 1) {
        r = spi_xchg(0xFF);
        if ((r & 0x80) == 0) return r;
    }
    return 0xFF;
}

int main(void) {
    int i;
    int b;
    int tries;

    // ---- Init: CS high, >=80 dummy clocks ----
    cs_high();
    for (i = 0; i < 10; i = i + 1) spi_xchg(0xFF);

    cs_low();

    // CMD0 (reset) -> R1 = 0x01 (idle)
    sd_cmd(0x40, 0, 0, 0, 0, 0x95);
    sd_r1();

    // CMD8 (voltage check) -> R1 + 4-byte echo
    sd_cmd(0x48, 0, 0, 0x01, 0xAA, 0x87);
    for (i = 0; i < 5; i = i + 1) spi_xchg(0xFF);

    // ACMD41 loop until ready (CMD55 + ACMD41)
    for (tries = 0; tries < 20; tries = tries + 1) {
        sd_cmd(0x77, 0, 0, 0, 0, 0x01);    // CMD55
        sd_r1();
        sd_cmd(0x69, 0x40, 0, 0, 0, 0x01); // ACMD41
        if (sd_r1() == 0) break;
    }

    // CMD16 (set block length 512)
    sd_cmd(0x50, 0, 0, 0x02, 0x00, 0x01);
    sd_r1();

    // CMD17 (read single block, sector 0)
    sd_cmd(0x51, 0, 0, 0, 0, 0x01);
    sd_r1();

    // Wait for data token 0xFE
    while (spi_xchg(0xFF) != 0xFE) {}

    // Read 512 bytes; print first 16 as hex pairs, discard rest
    for (i = 0; i < 16; i = i + 1) {
        b = spi_xchg(0xFF);
        print_hex(b);
        if (i < 15) putc(' '); else putc('\n');
    }
    for (i = 0; i < 496; i = i + 1) spi_xchg(0xFF);

    // 2 CRC bytes (discard)
    spi_xchg(0xFF);
    spi_xchg(0xFF);

    cs_high();
    spi_xchg(0xFF);    // trailing clock

    return 0;
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
