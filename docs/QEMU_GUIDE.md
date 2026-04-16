# AI-OS QEMU Testing Guide

This guide explains how to build and test AI-OS using QEMU virtualization.

## Prerequisites

### macOS
```bash
brew install qemu grub xorriso
```

### Ubuntu/Debian
```bash
sudo apt-get install qemu-system-x86 grub-pc-bin grub-common xorriso mtools
```

### Arch Linux
```bash
sudo pacman -S qemu grub xorriso
```

### Rust (all platforms)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
```

## Building the ISO

### Step 1: Build the kernel
```bash
cd kernel_rs
cargo build --release
```

### Step 2: Create bootable ISO
```bash
./build_iso.sh
```

This will create `aios.iso` in the current directory.

## Testing Methods

### Method 1: Full ISO Test (Recommended)
```bash
./run_qemu.sh
```

**Options:**
- `-m 4G` - Set memory to 4GB
- `-c 8` - Use 8 CPU cores
- `--headless` - Run without display
- `--vnc` - Use VNC (connect to localhost:5900)

**Examples:**
```bash
# Run with 4GB RAM and 8 CPUs
./run_qemu.sh -m 4G -c 8

# Run headless (no GUI)
./run_qemu.sh --headless

# Run with VNC
./run_qemu.sh --vnc
# Then connect with: vncviewer localhost:5900
```

### Method 2: Quick Kernel Test
```bash
./test_qemu_simple.sh
```

Tests the kernel binary directly without creating an ISO. Faster for development iterations.

### Method 3: Debug Mode
```bash
./qemu_debug.sh
```

Runs QEMU with GDB support. In another terminal:
```bash
gdb kernel_rs/target/x86_64-aios/release/aios_kernel
(gdb) target remote localhost:1234
(gdb) continue
```

## QEMU Controls

- **Release mouse/keyboard**: `Ctrl+Alt+G` (GTK mode)
- **Exit QEMU**: `Ctrl+A` then `X` (serial mode)
- **Switch to monitor**: `Ctrl+Alt+2`
- **Switch to console**: `Ctrl+Alt+1`

## Boot Process

When you run AI-OS in QEMU, you should see:

1. **GRUB menu** with these options:
   - AI-OS (normal boot)
   - AI-OS (Safe Mode)
   - AI-OS (Debug Mode)

2. **Kernel boot** messages showing:
   - Hardware initialization
   - Memory setup
   - Interrupt handlers
   - Device drivers

3. **System prompt** or GUI (depending on configuration)

## Troubleshooting

### "grub-mkrescue not found"
Install GRUB tools:
```bash
# macOS
brew install grub

# Linux
sudo apt-get install grub-pc-bin grub-common
```

### "qemu-system-x86_64 not found"
Install QEMU:
```bash
# macOS
brew install qemu

# Linux
sudo apt-get install qemu-system-x86
```

### "ISO build failed"
1. Ensure kernel is built:
   ```bash
   cd kernel_rs && cargo build --release
   ```

2. Check build directory permissions:
   ```bash
   rm -rf build/iso
   ```

3. Run build script with verbose output:
   ```bash
   bash -x ./build_iso.sh
   ```

### QEMU shows black screen
1. Check serial output (enabled by default with `-serial stdio`)
2. Verify kernel was built correctly
3. Try debug mode to see detailed logs

### KVM not available
The scripts automatically fall back to software emulation if KVM is not available. KVM provides better performance but requires:
- Linux host OS
- CPU with virtualization support (Intel VT-x or AMD-V)
- KVM kernel module loaded

## Advanced QEMU Options

### Network Testing
```bash
qemu-system-x86_64 \
    -cdrom aios.iso \
    -m 2G \
    -netdev user,id=net0,hostfwd=tcp::8080-:80 \
    -device e1000,netdev=net0
```

### Disk Image Testing
```bash
# Create a disk image
qemu-img create -f qcow2 aios_disk.qcow2 10G

# Run with disk
qemu-system-x86_64 \
    -cdrom aios.iso \
    -m 2G \
    -drive file=aios_disk.qcow2,format=qcow2
```

### GPU Testing (experimental)
```bash
qemu-system-x86_64 \
    -cdrom aios.iso \
    -m 2G \
    -vga virtio \
    -display sdl,gl=on
```

## Performance Tips

1. **Use KVM** when available (Linux only):
   - 10-50x faster than software emulation
   - Automatically enabled by scripts when available

2. **Allocate sufficient RAM**:
   - Minimum: 512MB
   - Recommended: 2GB
   - For ML workloads: 4GB+

3. **Use multiple CPUs**:
   - `-smp 4` for quad-core
   - `-smp $(nproc)` to match host CPU count

4. **Enable VirtIO** for better I/O:
   - Network: `-netdev user,id=net0 -device virtio-net-pci,netdev=net0`
   - Disk: `-drive file=disk.qcow2,if=virtio`

## Development Workflow

1. **Edit kernel code** in `kernel_rs/src/`
2. **Build kernel**: `cargo build --release --manifest-path kernel_rs/Cargo.toml`
3. **Quick test**: `./test_qemu_simple.sh`
4. **Full ISO test**: `./build_iso.sh && ./run_qemu.sh`

For rapid iteration, use the simple test during development and full ISO test for releases.

## CI/CD Integration

The included GitHub Actions workflow (`.github/workflows/build-iso.yml`) automatically:
- Builds the kernel
- Creates the ISO
- Runs QEMU tests
- Uploads ISO artifacts

Artifacts are retained for 30 days and can be downloaded from the Actions tab.

## Additional Resources

- [QEMU Documentation](https://www.qemu.org/docs/master/)
- [GRUB Manual](https://www.gnu.org/software/grub/manual/)
- [Multiboot2 Specification](https://www.gnu.org/software/grub/manual/multiboot2/)
- [OSDev Wiki](https://wiki.osdev.org/)
