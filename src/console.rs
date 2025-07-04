// Research findings: format_args! cannot be used in this environment
// Only macro-based implementation works reliably

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

// Number printing (supports 0-99)
#[macro_export]
macro_rules! print_number {
    ($n:expr) => {{
        let num = $n;
        if num == 0 {
            unsafe {
                core::ptr::write_volatile(UART0, b'0');
            }
        } else if num < 10 {
            unsafe {
                core::ptr::write_volatile(UART0, (num as u8) + b'0');
            }
        } else if num < 100 {
            let tens = (num / 10) as u8 + b'0';
            let ones = (num % 10) as u8 + b'0';
            unsafe {
                core::ptr::write_volatile(UART0, tens);
                core::ptr::write_volatile(UART0, ones);
            }
        } else {
            print!("??"); // Display ?? for numbers >= 100
        }
    }};
}

// Convenience macro for printing message + number
#[macro_export]
macro_rules! println_number {
    ($msg:expr, $num:expr) => {{
        print!($msg);
        print_number!($num);
        println!();
    }};
}
