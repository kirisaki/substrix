// UART0 address for QEMU virt machine

#[macro_export]
macro_rules! print {
    ($s:expr) => {
        for b in $s.bytes() {
            unsafe {
                core::ptr::write_volatile(UART0, b);
            }
        }
    };
}

#[macro_export]
macro_rules! println {
    () => {
        unsafe {
            core::ptr::write_volatile(UART0, b'\n');
        }
    };
    ($s:expr) => {{
        print!($s);
        unsafe {
            core::ptr::write_volatile(UART0, b'\n');
        }
    }};
}

// Enhanced number printing (supports up to 4294967295 for u32)
#[macro_export]
macro_rules! print_number {
    ($n:expr) => {{
        let num = $n;
        if num == 0 {
            unsafe {
                core::ptr::write_volatile(UART0, b'0');
            }
        } else {
            // Convert number to string buffer
            let mut buffer = [0u8; 10]; // Enough for u32 max value
            let mut temp = num;
            let mut pos = 0;

            // Extract digits in reverse order
            while temp > 0 {
                buffer[pos] = (temp % 10) as u8 + b'0';
                temp /= 10;
                pos += 1;
            }

            // Print digits in correct order
            while pos > 0 {
                pos -= 1;
                unsafe {
                    core::ptr::write_volatile(UART0, buffer[pos]);
                }
            }
        }
    }};
}

// Hexadecimal printing
#[macro_export]
macro_rules! print_hex {
    ($n:expr) => {{
        let num = $n;
        let hex_chars = b"0123456789abcdef";

        // Print "0x" prefix
        unsafe {
            core::ptr::write_volatile(UART0, b'0');
            core::ptr::write_volatile(UART0, b'x');
        }

        if num == 0 {
            unsafe {
                core::ptr::write_volatile(UART0, b'0');
            }
        } else {
            let mut buffer = [0u8; 16]; // Enough for u64 in hex
            let mut temp = num;
            let mut pos = 0;

            while temp > 0 {
                buffer[pos] = hex_chars[(temp % 16) as usize];
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
    }};
}

// Convenience macros
#[macro_export]
macro_rules! println_number {
    ($msg:expr, $num:expr) => {{
        print!($msg);
        print_number!($num);
        println!();
    }};
}

#[macro_export]
macro_rules! println_hex {
    ($msg:expr, $num:expr) => {{
        print!($msg);
        print_hex!($num);
        println!();
    }};
}

// Debug printing macro
#[macro_export]
macro_rules! debug {
    ($var:ident) => {{
        print!(stringify!($var));
        print!(" = ");
        print_number!($var);
        println!();
    }};
}

#[macro_export]
macro_rules! debug_hex {
    ($var:ident) => {{
        print!(stringify!($var));
        print!(" = ");
        print_hex!($var);
        println!();
    }};
}
