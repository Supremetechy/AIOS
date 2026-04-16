# AI-OS Bare-Metal Deployment Guide

## 🎯 Overview

Your AI-OS now has **bare-metal capability**! This means it can boot directly from BIOS/UEFI without requiring an underlying operating system.

## 📦 What's Included

### Boot Components
- **`boot/boot.asm`** - GRUB/Multiboot2 bootloader (Assembly)
- **`boot/boot_uefi.asm`** - UEFI bootloader (Assembly)

### Rust Kernel
- **`kernel_rs/src/main.rs`** - Kernel entry point
- **`kernel_rs/src/gdt.rs`** - Global Descriptor Table
- **`kernel_rs/src/interrupts.rs`** - Interrupt handling (IDT)
- **`kernel_rs/src/memory.rs`** - Memory management
- **`kernel_rs/src/allocator.rs`** - Heap allocator
- **`kernel_rs/src/vga_buffer.rs`** - VGA text mode driver
- **`kernel_rs/src/serial.rs`** - Serial port driver
- **`kernel_rs/src/task.rs`** - Task scheduler

### Device Drivers
- **`kernel_rs/src/drivers/pci.rs`** - PCI bus enumeration
- **`kernel_rs/src/drivers/gpu.rs`** - GPU detection (NVIDIA/AMD/Intel)
- **`kernel_rs/src/drivers/disk.rs`** - Disk drivers (ATA/SATA/NVMe)
- **`kernel_rs/src/drivers/network.rs`** - Network drivers (Ethernet/WiFi)

### Build System
- **`Makefile`** - Build automation
- **`build.sh`** - Build script with dependency checking
- **`kernel_rs/x86_64-aios.json`** - Rust target specification
- **`kernel_rs/linker.ld`** - Linker script

## 🚀 Building the OS

### Prerequisites

**Required:**
- NASM assembler
- Rust toolchain (rustc, cargo)
- Build tools (make, gcc/clang)

**Optional:**
- GRUB (grub-mkrescue) - for ISO creation
- QEMU - for testing

### Installation

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install nasm build-essential grub-pc-bin grub-common xorriso qemu-system-x86

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### macOS
```bash
brew install nasm qemu

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Arch Linux
```bash
sudo pacman -S nasm rust grub xorriso qemu
```

### Build Commands

**Simple build:**
```bash
./build.sh
```

**Using Make:**
```bash
make            # Build everything
make bootloader # Build bootloader only
make kernel     # Build kernel only
make iso        # Create bootable ISO
make run        # Test in QEMU
make clean      # Clean build files
```

## 💿 Creating Bootable Media

### USB Drive

```bash
# After building
sudo dd if=build/aios.iso of=/dev/sdX bs=4M status=progress
sudo sync
```

**⚠️ WARNING:** Replace `/dev/sdX` with your actual USB drive. This will erase all data!

### CD/DVD

```bash
# Burn ISO to disc
cdrecord -v dev=/dev/cdrom build/aios.iso
```

### Virtual Machine

#### QEMU
```bash
make run

# Or manually:
qemu-system-x86_64 -cdrom build/aios.iso -m 512M

# With KVM acceleration:
qemu-system-x86_64 -enable-kvm -cdrom build/aios.iso -m 512M
```

#### VirtualBox
1. Create new VM (Type: Linux, Version: Other Linux 64-bit)
2. Allocate 512MB+ RAM
3. Attach `build/aios.iso` as CD-ROM
4. Boot VM

#### VMware
1. Create new VM
2. Select "Install from ISO"
3. Choose `build/aios.iso`
4. Configure with 512MB+ RAM
5. Start VM

## 🖥️ Boot Process

### What Happens on Boot

```
1. BIOS/UEFI Power-On Self Test (POST)
   ↓
2. Boot device selection
   ↓
3. GRUB bootloader loads (from boot/boot.asm)
   ↓
4. Protected mode → Long mode (64-bit)
   ↓
5. Rust kernel loads (rust_main)
   ↓
6. Initialize GDT & IDT
   ↓
7. Set up memory management
   ↓
8. Load device drivers (PCI, GPU, Disk, Network)
   ↓
9. Initialize task scheduler
   ↓
10. Launch shell
    ↓
11. Ready for user input
```

### Boot Output

```
╔═══════════════════════════════════════════════════════════════╗
║                  AI-OS Bare-Metal Kernel                      ║
║                    Version 1.0.0-alpha                        ║
╚═══════════════════════════════════════════════════════════════╝

[KERNEL] Initializing AI-OS...
[KERNEL] Setting up memory management...
[MEMORY] Physical memory offset: 0x0
[MEMORY] ✓ Memory management initialized
[KERNEL] Setting up interrupt handlers...
[KERNEL] Loading device drivers...
[PCI] Scanning PCI bus...
[PCI] Found X device(s)
[GPU] Initializing GPU drivers...
[DISK] Initializing disk drivers...
[NETWORK] Initializing network drivers...
[DRIVERS] ✓ Device drivers initialized
[KERNEL] Starting task scheduler...
[TASK] ✓ Task scheduler initialized
[KERNEL] ✓ Boot complete!

AI-OS is ready. Type 'help' for available commands.

aios>
```

## 💻 User Interface

### Current Interface: Text-Based Console

**Input:**
- PS/2 Keyboard (automatic detection)
- Serial console (for remote/debugging)

**Output:**
- VGA text mode (80x25, color)
- Serial port (COM1 at 0x3F8)

### Available (When Implemented)
```
aios> help                    # Show commands
aios> status                  # System status
aios> hardware                # Hardware info
aios> gpus                    # List GPUs
aios> memory                  # Memory usage
aios> tasks                   # Running tasks
aios> disk                    # Disk info
aios> network                 # Network info
aios> resources               # Resource allocation
aios> onboarding              # Onboarding wizard
aios> specs                   # Detailed specs
aios> demo                   # AI-OS Demo
```

## 🔧 Hardware Support

### Detected Automatically

✅ **CPU**
- x86_64 architecture
- Multi-core support
- CPUID detection

✅ **Memory**
- Physical memory mapping
- Paging (4KB pages)
- Heap allocation

✅ **GPU** (via PCI)
- NVIDIA (Vendor 0x10DE)
- AMD (Vendor 0x1002)
- Intel (Vendor 0x8086)

✅ **Storage**
- ATA/SATA drives
- NVMe drives (basic)

✅ **Network**
- Ethernet controllers
- WiFi adapters (detected)

### Drivers Status

| Component | Detection | Driver | Status |
|-----------|-----------|---------|---------|
| CPU | ✅ Full | ✅ Full | Working |
| Memory | ✅ Full | ✅ Full | Working |
| VGA Display | ✅ Full | ✅ Full | Working |
| Keyboard | ✅ Full | ✅ Full | Working |
| Serial Port | ✅ Full | ✅ Full | Working |
| PCI Bus | ✅ Full | ✅ Basic | Working |
| GPU | ✅ Detection | 🚧 Stub | Planned |
| Disk | ✅ Detection | 🚧 Stub | Planned |
| Network | ✅ Detection | 🚧 Stub | Planned |

## 🐛 Debugging

### Serial Console

Connect via serial port to see debug output:

```bash
# In one terminal (if running in QEMU)
make run

# QEMU automatically shows serial output in the terminal
```

### QEMU Debug Mode

```bash
make debug

# In another terminal:
gdb kernel_rs/target/x86_64-aios/debug/aios_kernel
(gdb) target remote :1234
(gdb) continue
```

### Common Issues

**Build fails: "nasm: command not found"**
```bash
# Install NASM
sudo apt install nasm  # Ubuntu/Debian
brew install nasm      # macOS
```

**Build fails: "cargo: command not found"**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**No ISO created**
```bash
# Install GRUB tools
sudo apt install grub-pc-bin grub-common xorriso
```

**Kernel panics on boot**
- Check QEMU output for panic message
- Verify hardware requirements (64-bit CPU, 512MB+ RAM)
- Try with more memory: `qemu-system-x86_64 -m 1024M -cdrom build/aios.iso`

## 📊 System Requirements

### Minimum
- x86_64 CPU (64-bit)
- 512 MB RAM
- 100 MB disk space
- VGA-compatible display

### Recommended
- x86_64 CPU with 4+ cores
- 2+ GB RAM
- GPU (NVIDIA/AMD/Intel)
- 1+ GB disk space

## 🔮 Future Enhancements

### Phase 1 (Current)
- ✅ Bootloader (GRUB/UEFI)
- ✅ Kernel initialization
- ✅ Memory management
- ✅ Interrupt handling
- ✅ Basic drivers (VGA, Serial, Keyboard)
- ✅ Device detection (PCI, GPU, Disk)

### Phase 2 (Planned)
- 🚧 Full GPU drivers (CUDA/ROCm/OneAPI)
- 🚧 Filesystem support (ext4, FAT32)
- 🚧 Network stack (TCP/IP)
- 🚧 Process scheduler enhancement
- 🚧 Python interpreter embedding

### Phase 3 (Future)
- 📋 Multi-core SMP support
- 📋 Advanced memory management (NUMA)
- 📋 Container runtime integration
- 📋 Distributed training support
- 📋 Web-based management interface

## 🎓 Next Steps

1. **Build the OS**: `./build.sh`
2. **Test in QEMU**: `make run`
3. **Examine output**: Check boot messages
4. **Customize**: Modify kernel code in `kernel_rs/src/`
5. **Deploy**: Create USB or burn to CD

## 📚 Resources

- **Rust OSDev**: https://os.phil-opp.com/
- **OSDev Wiki**: https://wiki.osdev.org/
- **Multiboot2**: https://www.gnu.org/software/grub/manual/multiboot2/
- **x86_64 Crate**: https://docs.rs/x86_64/

## ⚠️ Important Notes

- This is a **bare-metal kernel** - it replaces your OS when booted
- **Test in a VM first** before deploying to real hardware
- **Backup data** before creating bootable USB
- Currently **no filesystem support** - changes are not persisted
- **No network yet** - planned for Phase 2

---

**Status**: ✅ Bare-metal bootable kernel complete!
**Interface**: Text-based console (VGA + Keyboard)
**Ready**: Yes, bootable from USB/CD/VM
