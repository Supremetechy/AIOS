# Quick Start: Install QEMU and GRUB

## macOS Installation

```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install QEMU and GRUB
brew install qemu grub xorriso

# Verify installation
qemu-system-x86_64 --version
grub-mkrescue --version
```

## Ubuntu/Debian Installation

```bash
# Update package list
sudo apt-get update

# Install required packages
sudo apt-get install -y \
    qemu-system-x86 \
    qemu-utils \
    grub-pc-bin \
    grub-common \
    xorriso \
    mtools

# Verify installation
qemu-system-x86_64 --version
grub-mkrescue --version
```

## Arch Linux Installation

```bash
# Install packages
sudo pacman -S qemu grub xorriso

# Verify installation
qemu-system-x86_64 --version
grub-mkrescue --version
```

## Fedora/RHEL Installation

```bash
# Install packages
sudo dnf install -y \
    qemu-system-x86 \
    grub2-tools-extra \
    xorriso \
    mtools

# Verify installation
qemu-system-x86_64 --version
grub2-mkrescue --version
```

## Rust Installation (All Platforms)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the environment
source $HOME/.cargo/env

# Install nightly toolchain and rust-src
rustup toolchain install nightly
rustup default nightly
rustup component add rust-src llvm-tools-preview

# Verify installation
rustc --version
cargo --version
```

## First Build and Test

```bash
# 1. Build the Rust kernel
cd kernel_rs
cargo build --release
cd ..

# 2. Create bootable ISO
./build_iso.sh

# 3. Test in QEMU
./run_qemu.sh
```

## Quick Test (Without ISO)

```bash
# Test kernel directly (faster for development)
./test_qemu_simple.sh
```

## Troubleshooting

### grub-mkrescue: command not found (macOS)
```bash
# GRUB installation can be tricky on macOS
brew install grub
brew install xorriso

# If still not working, try:
brew reinstall grub
```

### Permission denied
```bash
chmod +x build_iso.sh run_qemu.sh test_qemu_simple.sh qemu_debug.sh
```

### Rust toolchain issues
```bash
# Ensure you're using nightly
rustup default nightly

# Update everything
rustup update
```

### Build errors in kernel_rs
```bash
# Clean and rebuild
cd kernel_rs
cargo clean
cargo build --release
```

## What's Next?

After successful installation:

1. Read `QEMU_GUIDE.md` for detailed testing options
2. Check `BARE_METAL_GUIDE.md` for running on real hardware
3. See `GETTING_STARTED.md` for development workflow

## System Requirements

**Minimum:**
- 2 CPU cores
- 4 GB RAM
- 5 GB disk space

**Recommended:**
- 4+ CPU cores
- 8 GB RAM
- 10 GB disk space
- Hardware virtualization support (Intel VT-x / AMD-V)
