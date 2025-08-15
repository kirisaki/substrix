//! Enhanced Console Output System
//!
//! This module provides a comprehensive console output system for bare-metal
//! RISC-V systems, including format macro support, safe output functions,
//! and panic-safe emergency output capabilities.
//!
//! The module supports:
//! - Basic string output
//! - Numeric formatting (decimal and hexadecimal)
//! - Format macro support with `{}` placeholders
//! - Emergency output for panic situations
//! - Type-safe output functions

use crate::UART0;

/// Output a single byte to the UART console
///
/// This is the fundamental output function that all other console
/// functions build upon. It directly writes to the UART hardware.
///
/// # Arguments
/// * `c` - The byte to output
///
/// # Safety
/// This function performs raw hardware access but is safe to call
/// as it only writes to the output register.
#[inline]
pub fn put_char(c: u8) {
    unsafe {
        core::ptr::write_volatile(UART0, c);
    }
}

/// Output a string slice to the console
///
/// Outputs each byte of the string sequentially to the UART.
/// Handles UTF-8 strings by outputting the raw bytes.
///
/// # Arguments
/// * `s` - The string slice to output
///
/// # Examples
/// ```rust
/// put_str("Hello, World!");
/// ```
pub fn put_str(s: &str) {
    for byte in s.bytes() {
        put_char(byte);
    }
}

/// Output a newline character to the console
///
/// Outputs a Unix-style line feed character (0x0A).
pub fn put_newline() {
    put_char(b'\n');
}

/// Output an unsigned 64-bit number in decimal format
///
/// Converts the number to decimal representation and outputs
/// each digit. Handles the special case of zero correctly.
///
/// # Arguments
/// * `num` - The number to output
///
/// # Examples
/// ```rust
/// put_number(42);     // Outputs: "42"
/// put_number(0);      // Outputs: "0"
/// put_number(12345);  // Outputs: "12345"
/// ```
pub fn put_number(num: u64) {
    if num == 0 {
        put_char(b'0');
        return;
    }

    let mut buffer = [0u8; 20]; // Enough for u64::MAX
    let mut temp = num;
    let mut pos = 0;

    // Build number string in reverse
    while temp > 0 {
        buffer[pos] = (temp % 10) as u8 + b'0';
        temp /= 10;
        pos += 1;
    }

    // Output digits in correct order
    while pos > 0 {
        pos -= 1;
        put_char(buffer[pos]);
    }
}

/// Output a number in hexadecimal format with '0x' prefix
///
/// Converts the number to hexadecimal representation with lowercase
/// letters (a-f) and includes the standard '0x' prefix.
///
/// # Arguments
/// * `num` - The number to output in hexadecimal
///
/// # Examples
/// ```rust
/// put_hex(255);    // Outputs: "0xff"
/// put_hex(0);      // Outputs: "0x0"
/// put_hex(4096);   // Outputs: "0x1000"
/// ```
pub fn put_hex(num: usize) {
    const HEX_CHARS: &[u8] = b"0123456789abcdef";

    put_str("0x");

    if num == 0 {
        put_char(b'0');
        return;
    }

    let mut buffer = [0u8; 16]; // Enough for usize on 64-bit
    let mut temp = num;
    let mut pos = 0;

    // Build hex string in reverse
    while temp > 0 {
        buffer[pos] = HEX_CHARS[temp % 16];
        temp /= 16;
        pos += 1;
    }

    // Output hex digits in correct order
    while pos > 0 {
        pos -= 1;
        put_char(buffer[pos]);
    }
}

/// Format argument types for the simple format system
#[derive(Clone, Copy)]
pub enum FormatArg {
    /// String argument
    Str(&'static str),
    /// Unsigned 64-bit number argument
    Number(u64),
    /// Hexadecimal number argument
    Hex(usize),
}

/// Simple format string processor
///
/// Processes a format string with `{}` placeholders and replaces them
/// with the provided arguments. This is a simplified version of Rust's
/// format system suitable for no_std environments.
///
/// # Arguments
/// * `format_str` - The format string containing `{}` placeholders
/// * `args` - Slice of format arguments to substitute
///
/// # Examples
/// ```rust
/// let args = [FormatArg::Str("world"), FormatArg::Number(42)];
/// put_format("Hello, {}! The answer is {}.", &args);
/// // Outputs: "Hello, world! The answer is 42."
/// ```
pub fn put_format(format_str: &str, args: &[FormatArg]) {
    let mut arg_index = 0;
    let mut chars = format_str.chars();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if let Some(next_ch) = chars.next() {
                if next_ch == '}' {
                    // Found a {} placeholder
                    if arg_index < args.len() {
                        match args[arg_index] {
                            FormatArg::Str(s) => put_str(s),
                            FormatArg::Number(n) => put_number(n),
                            FormatArg::Hex(h) => put_hex(h),
                        }
                        arg_index += 1;
                    } else {
                        // No more arguments, output placeholder as-is
                        put_str("{}");
                    }
                } else {
                    // Not a valid placeholder, output literal characters
                    put_char(b'{');
                    put_char(next_ch as u8);
                }
            } else {
                // End of string after {
                put_char(b'{');
            }
        } else {
            // Regular character
            put_char(ch as u8);
        }
    }
}

/// Enhanced print macro with format support
///
/// Supports both simple string output and format strings with arguments.
/// For format strings, use helper functions: num(), hex(), str() to wrap arguments.
///
/// # Examples
/// ```rust
/// print!("Hello");                          // Simple string
/// print!("Number: {}", num(42));            // With number  
/// print!("Hex: {}", hex(255));              // With hex
/// print!("Text: {}", str("hello"));         // With string
/// ```
#[macro_export]
macro_rules! print {
    // Simple string case
    ($s:expr) => {
        $crate::console::put_str($s)
    };

    // Format string with arguments
    ($fmt:expr, $($arg:expr),+) => {{
        let args = [$($arg),+];
        $crate::console::put_format($fmt, &args);
    }};
}

/// Enhanced println macro with format support
///
/// Like `print!` but adds a newline at the end.
/// For format strings, use helper functions: num(), hex(), str() to wrap arguments.
///
/// # Examples
/// ```rust
/// println!();                               // Just newline
/// println!("Hello");                        // Simple string with newline
/// println!("Number: {}", num(42));          // With number and newline
/// println!("Text: {}", str("hello"));       // With string and newline
/// ```
#[macro_export]
macro_rules! println {
    // Empty case - just newline
    () => {
        $crate::console::put_newline()
    };

    // Simple string case
    ($s:expr) => {{
        $crate::console::put_str($s);
        $crate::console::put_newline()
    }};

    // Format string with arguments
    ($fmt:expr, $($arg:expr),+) => {{
        let args = [$($arg),+];
        $crate::console::put_format($fmt, &args);
        $crate::console::put_newline();
    }};
}

/// Convert a number to hexadecimal format argument
///
/// Helper function to create hex format arguments.
///
/// # Arguments
/// * `num` - The number to format as hexadecimal
///
/// # Returns
/// A FormatArg::Hex variant containing the number
///
/// # Examples
/// ```rust
/// println!("Address: {}", hex(0x1000));  // Outputs: "Address: 0x1000"
/// ```
pub fn hex(num: usize) -> FormatArg {
    FormatArg::Hex(num)
}

/// Convert a number to decimal format argument
///
/// Helper function to create number format arguments.
///
/// # Arguments
/// * `num` - The number to format as decimal
///
/// # Returns
/// A FormatArg::Number variant containing the number
///
/// # Examples
/// ```rust
/// println!("Count: {}", num(42));  // Outputs: "Count: 42"
/// ```
pub fn num(number: u64) -> FormatArg {
    FormatArg::Number(number)
}

/// Convert a string to string format argument
///
/// Helper function to create string format arguments.
///
/// # Arguments
/// * `s` - The string to format
///
/// # Returns
/// A FormatArg::Str variant containing the string
///
/// # Examples
/// ```rust
/// println!("Message: {}", str("hello"));  // Outputs: "Message: hello"
/// ```
pub fn str(s: &'static str) -> FormatArg {
    FormatArg::Str(s)
}

/// Additional helper functions for different numeric types

/// Convert usize to format argument
pub fn num_usize(number: usize) -> FormatArg {
    FormatArg::Number(number as u64)
}

/// Convert u32 to format argument  
pub fn num_u32(number: u32) -> FormatArg {
    FormatArg::Number(number as u64)
}

/// Convert u16 to format argument
pub fn num_u16(number: u16) -> FormatArg {
    FormatArg::Number(number as u64)
}

/// Convert u8 to format argument
pub fn num_u8(number: u8) -> FormatArg {
    FormatArg::Number(number as u64)
}

// Legacy compatibility macros (simplified versions)

/// Legacy macro for number output (deprecated - use println! with format)
///
/// # Examples
/// ```rust
/// print_number!(42);  // Outputs: "42"
/// ```
#[macro_export]
macro_rules! print_number {
    ($n:expr) => {
        $crate::console::put_number($n)
    };
}

/// Legacy macro for hex output (deprecated - use println! with hex())
///
/// # Examples
/// ```rust
/// print_hex!(255);  // Outputs: "0xff"
/// ```
#[macro_export]
macro_rules! print_hex {
    ($n:expr) => {
        $crate::console::put_hex($n)
    };
}

/// Legacy macro for number output with message (deprecated)
///
/// # Examples
/// ```rust
/// println_number!("Value: ", 42);  // Use println!("Value: {}", num(42)) instead
/// ```
#[macro_export]
macro_rules! println_number {
    ($msg:expr, $num:expr) => {{
        $crate::console::put_str($msg);
        $crate::console::put_number($num);
        $crate::console::put_newline();
    }};
}

/// Legacy macro for hex output with message (deprecated)
///
/// # Examples
/// ```rust
/// println_hex!("Address: ", 0x1000);  // Use println!("Address: {}", hex(0x1000)) instead
/// ```
#[macro_export]
macro_rules! println_hex {
    ($msg:expr, $num:expr) => {{
        $crate::console::put_str($msg);
        $crate::console::put_hex($num);
        $crate::console::put_newline();
    }};
}

/// Legacy debug macro for variables (deprecated)
///
/// # Examples
/// ```rust
/// let value = 42;
/// debug!(value);  // Use println!("value = {}", num(value)) instead
/// ```
#[macro_export]
macro_rules! debug {
    ($var:ident) => {{
        $crate::console::put_str(stringify!($var));
        $crate::console::put_str(" = ");
        $crate::console::put_number($var);
        $crate::console::put_newline();
    }};
}

/// Legacy debug macro for hex variables (deprecated)
///
/// # Examples
/// ```rust
/// let addr = 0x1000;
/// debug_hex!(addr);  // Use println!("addr = {}", hex(addr)) instead
/// ```
#[macro_export]
macro_rules! debug_hex {
    ($var:ident) => {{
        $crate::console::put_str(stringify!($var));
        $crate::console::put_str(" = ");
        $crate::console::put_hex($var);
        $crate::console::put_newline();
    }};
}

// Panic-safe emergency output functions
// These functions are designed to work even in panic situations

/// Emergency string output for panic situations
///
/// This function bypasses normal safety checks and directly writes
/// to the UART. Used only during panic handling.
///
/// # Arguments
/// * `s` - The string to output
///
/// # Safety
/// This function is designed to be as robust as possible during
/// system failure conditions.
pub fn panic_put_str_safe(s: &str) {
    for byte in s.bytes() {
        unsafe {
            core::ptr::write_volatile(UART0, byte);
        }
    }
}

/// Emergency newline output for panic situations
///
/// Outputs a newline character directly to UART during panic conditions.
pub fn panic_put_newline_safe() {
    unsafe {
        core::ptr::write_volatile(UART0, b'\n');
    }
}

/// Emergency number output for panic situations
///
/// Outputs a number in decimal format directly to UART during panic conditions.
///
/// # Arguments
/// * `num` - The number to output
pub fn panic_put_number_safe(num: u64) {
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

/// Emergency hex output for panic situations
///
/// Outputs a number in hexadecimal format directly to UART during panic conditions.
///
/// # Arguments
/// * `num` - The number to output in hexadecimal
pub fn panic_put_hex_safe(num: usize) {
    const HEX_CHARS: &[u8] = b"0123456789abcdef";

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
        buffer[pos] = HEX_CHARS[temp % 16];
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

// Panic-safe macros (maintain compatibility)

/// Panic-safe print macro
///
/// For use in panic handlers where normal macros might not work.
#[macro_export]
macro_rules! panic_print {
    ($s:expr) => {
        $crate::console::panic_put_str_safe($s)
    };
}

/// Panic-safe println macro
///
/// For use in panic handlers where normal macros might not work.
#[macro_export]
macro_rules! panic_println {
    () => {
        $crate::console::panic_put_newline_safe()
    };
    ($s:expr) => {{
        $crate::console::panic_put_str_safe($s);
        $crate::console::panic_put_newline_safe()
    }};
}

/// Panic-safe number output macro
#[macro_export]
macro_rules! panic_print_number {
    ($n:expr) => {
        $crate::console::panic_put_number_safe($n)
    };
}

/// Panic-safe hex output macro
#[macro_export]
macro_rules! panic_print_hex {
    ($n:expr) => {
        $crate::console::panic_put_hex_safe($n)
    };
}

/// Console system test function
///
/// Tests all console functionality including format macros,
/// number output, and hex output.
pub fn test_console_system() {
    println!("=== CONSOLE SYSTEM TEST ===");

    // Test basic output
    println!("Basic string output works");

    // Test number formatting
    println!("Number test: {}", num(42));
    println!("Hex test: {}", hex(255));

    // Test legacy macros for compatibility
    print!("Legacy compatibility: ");
    print_number!(456);
    print!(" and ");
    print_hex!(4096);
    println!();

    println!("✓ Console system test completed");
}
