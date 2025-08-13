# Substrix

A RISC-V unikernel written in Rust for bare-metal systems.

## Features

- RISC-V 64-bit unikernel
- UART console output
- Basic trap handling
- QEMU virt machine compatible

## Requirements

- Rust toolchain with `riscv64gc-unknown-none-elf` target
- QEMU (qemu-system-riscv64)

## Setup

```bash
# Add RISC-V target
rustup target add riscv64gc-unknown-none-elf

# Install QEMU (Ubuntu/Debian)
sudo apt install qemu-system-misc
```

## Build & Run

```bash
# Build
cargo build --release

# Run
cargo run --release
```

## Current Status

- ✅ Basic UART output
- ✅ Trap handling (`ecall` support)  
- 🚧 Timer interrupts (in progress)
- ⏳ System calls
- ⏳ Process management

## License

Dual licensed under Apache 2.0 and MIT licenses.

## Author

Akihito Kirisaki