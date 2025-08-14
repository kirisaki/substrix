// Fixed Console Macros - src/console.rs
// マクロの式コンテキスト問題を修正

use crate::UART0;

/// 単一バイトをUARTに出力
#[inline]
pub fn put_char(c: u8) {
    unsafe {
        core::ptr::write_volatile(UART0, c);
    }
}

/// 文字列をUARTに出力
pub fn put_str(s: &str) {
    for byte in s.bytes() {
        put_char(byte);
    }
}

/// 改行を出力
pub fn put_newline() {
    put_char(b'\n');
}

/// 数値を文字列として出力
pub fn put_number(num: u64) {
    if num == 0 {
        put_char(b'0');
        return;
    }

    let mut buffer = [0u8; 20];
    let mut temp = num;
    let mut pos = 0;

    while temp > 0 {
        buffer[pos] = (temp % 10) as u8 + b'0';
        temp /= 10;
        pos += 1;
    }

    while pos > 0 {
        pos -= 1;
        put_char(buffer[pos]);
    }
}

/// 16進数を出力
pub fn put_hex(num: usize) {
    let hex_chars = b"0123456789abcdef";

    put_char(b'0');
    put_char(b'x');

    if num == 0 {
        put_char(b'0');
        return;
    }

    let mut buffer = [0u8; 16];
    let mut temp = num;
    let mut pos = 0;

    while temp > 0 {
        buffer[pos] = hex_chars[temp % 16];
        temp /= 16;
        pos += 1;
    }

    while pos > 0 {
        pos -= 1;
        put_char(buffer[pos]);
    }
}

// 修正されたマクロ（$crateを使わない形）
#[macro_export]
macro_rules! print {
    ($s:expr) => {
        crate::console::put_str($s)
    };
}

#[macro_export]
macro_rules! println {
    () => {
        crate::console::put_newline()
    };
    ($s:expr) => {{
        crate::console::put_str($s);
        crate::console::put_newline()
    }};
}

#[macro_export]
macro_rules! print_number {
    ($n:expr) => {
        crate::console::put_number($n)
    };
}

#[macro_export]
macro_rules! print_hex {
    ($n:expr) => {
        crate::console::put_hex($n)
    };
}

// 便利なマクロ
#[macro_export]
macro_rules! println_number {
    ($msg:expr, $num:expr) => {{
        crate::console::put_str($msg);
        crate::console::put_number($num);
        crate::console::put_newline();
    }};
}

#[macro_export]
macro_rules! println_hex {
    ($msg:expr, $num:expr) => {{
        crate::console::put_str($msg);
        crate::console::put_hex($num);
        crate::console::put_newline();
    }};
}

// デバッグマクロ
#[macro_export]
macro_rules! debug {
    ($var:ident) => {{
        crate::console::put_str(stringify!($var));
        crate::console::put_str(" = ");
        crate::console::put_number($var);
        crate::console::put_newline();
    }};
}

#[macro_export]
macro_rules! debug_hex {
    ($var:ident) => {{
        crate::console::put_str(stringify!($var));
        crate::console::put_str(" = ");
        crate::console::put_hex($var);
        crate::console::put_newline();
    }};
}

// パニック専用の安全な出力関数
/// パニック専用の安全な文字列出力
pub fn print_panic_str_safe(s: &str) {
    for byte in s.bytes() {
        unsafe {
            core::ptr::write_volatile(UART0, byte);
        }
    }
}

/// パニック専用の安全な改行出力
pub fn print_panic_newline_safe() {
    unsafe {
        core::ptr::write_volatile(UART0, b'\n');
    }
}

/// パニック専用の数値出力
pub fn print_panic_number_safe(num: u64) {
    if num == 0 {
        unsafe {
            core::ptr::write_volatile(UART0, b'0');
        }
        return;
    }

    let mut buffer = [0u8; 20];
    let mut temp = num;
    let mut pos = 0;

    while temp > 0 {
        buffer[pos] = (temp % 10) as u8 + b'0';
        temp /= 10;
        pos += 1;
    }

    while pos > 0 {
        pos -= 1;
        unsafe {
            core::ptr::write_volatile(UART0, buffer[pos]);
        }
    }
}

/// パニック専用の16進数出力
pub fn print_panic_hex_safe(num: usize) {
    let hex_chars = b"0123456789abcdef";

    unsafe {
        core::ptr::write_volatile(UART0, b'0');
        core::ptr::write_volatile(UART0, b'x');
    }

    if num == 0 {
        unsafe {
            core::ptr::write_volatile(UART0, b'0');
        }
        return;
    }

    let mut buffer = [0u8; 16];
    let mut temp = num;
    let mut pos = 0;

    while temp > 0 {
        buffer[pos] = hex_chars[temp % 16];
        temp /= 16;
        pos += 1;
    }

    while pos > 0 {
        pos -= 1;
        unsafe {
            core::ptr::write_volatile(UART0, buffer[pos]);
        }
    }
}

// パニック用のマクロ（関数を直接呼び出す形）
#[macro_export]
macro_rules! panic_print {
    ($s:expr) => {
        crate::console::print_panic_str_safe($s)
    };
}

#[macro_export]
macro_rules! panic_println {
    () => {
        crate::console::print_panic_newline_safe()
    };
    ($s:expr) => {{
        crate::console::print_panic_str_safe($s);
        crate::console::print_panic_newline_safe()
    }};
}

#[macro_export]
macro_rules! panic_print_number {
    ($n:expr) => {
        crate::console::print_panic_number_safe($n)
    };
}

#[macro_export]
macro_rules! panic_print_hex {
    ($n:expr) => {
        crate::console::print_panic_hex_safe($n)
    };
}
