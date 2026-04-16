# AI-OS Build and Test Guide

Quick reference for building and testing AI-OS with QEMU.

## One-Command Quick Start

```bash
# Install tools, build kernel, create ISO, and test
make install && make test
```

## Step-by-Step

### 1. Install Prerequisites

```bash
make install
```

Or manually follow [INSTALL_QEMU.md](INSTALL_QEMU.md).

### 2. Build the Kernel

```bash
make kernel
```

This compiles the Rust kernel to `kernel_rs/target/x86_64-aios/release/aios_kernel`.

### 3. Create Bootable ISO

```bash
make iso
```

This creates `aios.iso` with GRUB bootloader.

### 4. Test in QEMU

```bash
# Full ISO test
make test

# Quick test (faster, no ISO)
make test-simple

# Debug mode (with GDB)
make test-debug
```

## Available Make Targets

| Target | Description |
|--------|-------------|
| `make kernel` | Build Rust kernel only |
| `make iso` | Build bootable ISO (default) |
| `make test` | Build and test ISO in QEMU |
| `make test-simple` | Quick kernel test (no ISO) |
| `make test-debug` | Run with GDB debugging |
| `make clean` | Remove build artifacts |
| `make install` | Install QEMU and GRUB |
| `make help` | Show all targets |

## Direct Script Usage

If you prefer not to use Make:

```bash
# Build ISO
./build_iso.sh

# Test with QEMU
./run_qemu.sh

# Quick test
./test_qemu_simple.sh

# Debug mode
./qemu_debug.sh
```

## QEMU Options

```bash
# Custom memory and CPU
./run_qemu.sh -m 4G -c 8

# Headless mode
./run_qemu.sh --headless

# VNC display
./run_qemu.sh --vnc
```

## Development Workflow

**Fast iteration** (recommended during development):
```bash
# Edit code in kernel_rs/src/
make test-simple
```

**Full test** (before commits):
```bash
make clean
make test
```

## Troubleshooting

**Build fails:**
```bash
make clean
cd kernel_rs && cargo clean && cd ..
make kernel
```

**QEMU shows black screen:**
- Check serial output (should show in terminal)
- Try debug mode: `make test-debug`

**Permission errors:**
```bash
chmod +x *.sh
```

## Next Steps

- Read [QEMU_GUIDE.md](QEMU_GUIDE.md) for advanced QEMU usage
- See [BARE_METAL_GUIDE.md](BARE_METAL_GUIDE.md) for real hardware
- Check [GETTING_STARTED.md](GETTING_STARTED.md) for development

## Quick Reference Card

```
┌─────────────────────────────────────┐
│ AI-OS Quick Commands                │
├─────────────────────────────────────┤
│ First time setup:                   │
│   make install                      │
│                                     │
│ Build and test:                     │
│   make test                         │
│                                     │
│ Fast dev loop:                      │
│   make test-simple                  │
│                                     │
│ Clean rebuild:                      │
│   make clean && make test           │
│                                     │
│ Debug with GDB:                     │
│   make test-debug                   │
│   # In another terminal:            │
│   gdb kernel_rs/target/.../aios_... │
│   (gdb) target remote localhost:1234│
└─────────────────────────────────────┘
```
