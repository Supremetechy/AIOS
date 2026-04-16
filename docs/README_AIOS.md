# AI-OS - Operating System for AI Workloads

A standalone operating system designed specifically for AI-based applications, providing comprehensive hardware detection, system management, and optimization for AI workloads.

## Features

### 🔍 Comprehensive Hardware Detection
- **CPU Detection**: Full CPU identification with cores, threads, frequency, and capabilities
- **GPU Detection**: Support for NVIDIA (CUDA), AMD (ROCm), Intel (OneAPI), and Apple (Metal) GPUs
- **TPU Detection**: Google Cloud TPU and Coral Edge TPU detection
- **NPU/VPU Detection**: Neural Processing Units and Vision Processing Units
- **System Specs**: Complete memory, storage, and network information

### 🚀 Boot System
- Multi-stage boot sequence (Firmware → Bootloader → Kernel → Drivers → Services)
- Safe mode support
- Hardware-aware driver initialization
- Automatic driver loading for detected accelerators (CUDA, ROCm, Metal, etc.)

### 💾 Filesystem Management
- Support for multiple filesystem types (ext4, XFS, BTRFS, NTFS, etc.)
- AI-optimized directory structure:
  - `/aios/models` - AI model storage
  - `/aios/datasets` - Training datasets
  - `/aios/cache` - Model cache
  - `/aios/var/checkpoints` - Training checkpoints
- Disk usage monitoring and space checks
- File operations with metadata tracking

### 🌐 Network Stack
- Automatic network interface detection
- TCP/IP stack initialization
- DNS configuration management
- Routing table management
- Network connectivity testing
- AI-specific port recommendations (TensorBoard, Jupyter, MLflow, etc.)

### 📊 System Monitoring
- Real-time CPU, GPU, memory, and disk monitoring
- GPU-specific metrics (VRAM, temperature, power usage, fan speed)
- Load average tracking
- Network statistics
- Continuous monitoring mode

## Installation

### Quick Setup with Onboarding GUI

For new users, we recommend using the interactive onboarding wizard:

```bash
# Install GUI dependencies
pip install -r requirements_gui.txt

# Run onboarding wizard
python run_onboarding.py
```

The onboarding wizard features:
- AI talking head videos guiding you through setup
- Automatic hardware detection and optimization
- Step-by-step configuration
- GPU acceleration setup

See [ONBOARDING_GUIDE.md](ONBOARDING_GUIDE.md) for detailed information.

### Manual Installation

```bash
# Clone the repository
git clone <repository-url>
cd ai-os

# No additional dependencies for basic functionality
# Optional: Install for GPU monitoring
pip install nvidia-ml-py3  # For NVIDIA GPUs
```

## Usage

### Quick Start

```bash
# Run the full OS kernel
python3 aios_kernel.py

# Monitor-only mode (quick system check)
python3 aios_kernel.py --monitor-only

# Safe mode boot
python3 aios_kernel.py --safe-mode

# Verbose mode
python3 aios_kernel.py --verbose
```

### Interactive Shell Commands

Once the kernel is running, you can use these commands:

```
aios> help        # Show all available commands
aios> status      # Show current system status
aios> monitor     # Start continuous monitoring
aios> hardware    # Show hardware information
aios> network     # Show network configuration
aios> filesystem  # Show filesystem information
aios> specs       # Show detailed system specifications
aios> exit        # Shutdown the system
```

### Individual Module Testing

```bash
# Test hardware detection
python3 kernel/hardware_detection.py

# Test boot system
python3 kernel/boot.py

# Test filesystem manager
python3 kernel/filesystem.py

# Test network manager
python3 kernel/network.py

# Test system monitor
python3 kernel/system_monitor.py
```

## Architecture

### Kernel Components

```
aios_kernel.py              # Main kernel entry point
├── kernel/
│   ├── __init__.py         # Kernel package initialization
│   ├── hardware_detection.py  # Hardware detection (CPU/GPU/TPU/NPU)
│   ├── boot.py             # Boot sequence manager
│   ├── filesystem.py       # Filesystem operations
│   ├── network.py          # Network stack
│   └── system_monitor.py   # Real-time monitoring
```

### Boot Sequence

1. **Power On** - System initialization
2. **Firmware** - UEFI/BIOS simulation
3. **Bootloader** - Configuration loading
4. **Kernel Load** - Kernel initialization
5. **Hardware Detection** - Identify all processors and devices
6. **Driver Initialization** - Load hardware-specific drivers
7. **Filesystem Mount** - Mount all filesystems
8. **Network Initialization** - Bring up network interfaces
9. **Service Start** - Start AI runtime services
10. **Ready** - System operational

## Hardware Support

### Processors

| Type | Vendors | Detection Method |
|------|---------|------------------|
| CPU | Intel, AMD, ARM, Apple | `/proc/cpuinfo`, `sysctl` |
| GPU | NVIDIA, AMD, Intel, Apple | `nvidia-smi`, `rocm-smi`, `lspci` |
| TPU | Google | Environment vars, `lsusb` |
| NPU/VPU | Intel Movidius, Qualcomm | `lsusb`, device enumeration |

### Capabilities Detection

- **NVIDIA**: CUDA, cuDNN, TensorRT support
- **AMD**: ROCm, HIP support
- **Intel**: OneAPI, Level Zero support
- **Apple**: Metal, Neural Engine support
- **Google TPU**: TensorFlow, JAX, PyTorch/XLA support

## System Requirements

### Minimum
- Any x86_64, ARM, or Apple Silicon processor
- 2 GB RAM
- 10 GB disk space
- Linux, macOS, or Windows

### Recommended for AI Workloads
- Multi-core CPU (8+ cores)
- 16+ GB RAM
- GPU with 8+ GB VRAM (NVIDIA, AMD, or Apple)
- 100+ GB SSD storage
- High-speed network connection

## Output Example

```
╔══════════════════════════════════════════════════════════════════════════╗
║                          AI-OS Boot Loader                               ║
║                    Operating System for AI Workloads                     ║
║                         Version 1.0.0-alpha                              ║
╚══════════════════════════════════════════════════════════════════════════╝

[BOOT] Starting boot sequence at 2026-04-01 22:56:25

[BOOT] [0.00s] Stage: FIRMWARE
[BOOT] Initializing firmware...
[BOOT] POST (Power-On Self-Test) - OK

[BOOT] [0.10s] Stage: HARDWARE_DETECT
[BOOT] Detecting CPU...
[BOOT] Detecting GPUs...
[BOOT] Detected 2 processor(s):
[BOOT]   - CPU: intel Intel(R) Core(TM) i9-9900K
[BOOT]   - GPU: nvidia NVIDIA GeForce RTX 3090
[BOOT] System Memory: 64.00 GB
[BOOT] ✓ GPU acceleration available

[BOOT] ✓ AI-OS boot complete in 2.34 seconds

[SYSTEM] Ready for AI workloads

[CAPABILITIES]
  ✓ GPU Acceleration: 1 device(s)
  ✓ CPU Processing: 1 processor(s)
  ✓ System Memory: 64.00 GB
```

## AI-Specific Features

### Model Storage Management
```python
from kernel.filesystem import AIFileSystemUtils

# Get recommended paths
model_path = AIFileSystemUtils.get_model_storage_path()
dataset_path = AIFileSystemUtils.get_dataset_storage_path()
cache_path = AIFileSystemUtils.get_cache_path()

# Estimate model size
size_gb = AIFileSystemUtils.estimate_model_size("transformer", 175000)  # 175B params

# Check available space
has_space = AIFileSystemUtils.check_model_storage_space(size_gb)
```

### Network Configuration for AI Services
```python
from kernel.network import AINetworkUtils

# Get recommended ports
ports = AINetworkUtils.get_recommended_ports()
# {
#   'inference_api': 8000,
#   'training_api': 8001,
#   'tensorboard': 6006,
#   'jupyter': 8888,
#   ...
# }

# Check port availability
available = AINetworkUtils.check_port_available(8000)

# Get distributed training config
config = AINetworkUtils.get_distributed_training_config()
```

## Command-Line Options

```
usage: aios_kernel.py [-h] [--safe-mode] [--verbose] [--no-network] [--no-fs] [--monitor-only]

AI-OS - Operating System for AI Workloads

optional arguments:
  -h, --help       show this help message and exit
  --safe-mode      Boot in safe mode
  --verbose, -v    Verbose output
  --no-network     Disable network initialization
  --no-fs          Skip filesystem auto-mount
  --monitor-only   Run system monitor only (no full boot)
```

## Development

### Adding New Hardware Support

1. Add detection logic to `kernel/hardware_detection.py`
2. Update `ProcessorType` enum if needed
3. Implement vendor-specific detection method
4. Add driver initialization in `kernel/boot.py`

### Extending Functionality

- **Custom Filesystems**: Extend `FileSystemType` in `kernel/filesystem.py`
- **Network Protocols**: Add to `NetworkProtocol` in `kernel/network.py`
- **Monitoring Metrics**: Extend dataclasses in `kernel/system_monitor.py`

## Troubleshooting

### No GPU Detected
- Ensure GPU drivers are installed (`nvidia-smi`, `rocm-smi`)
- Check that the GPU is properly seated and powered
- Verify CUDA/ROCm installation

### Network Not Starting
- Check network interfaces: `ip addr` or `ifconfig`
- Verify DNS configuration in `/etc/resolv.conf`
- Run with `--verbose` for detailed logs

### Filesystem Issues
- Check mount permissions
- Verify disk space: `df -h`
- Ensure directories are accessible

## Future Enhancements

- [ ] NUMA-aware memory allocation
- [ ] GPU process scheduling
- [ ] Model loading optimization
- [ ] Distributed training coordination
- [ ] Container runtime integration (Docker/Kubernetes)
- [ ] Resource quotas and limits
- [ ] Multi-tenant support
- [ ] Hot-plug device detection
- [ ] Power management for AI accelerators

## License

[Your License Here]

## Contributing

Contributions are welcome! Please feel free to submit pull requests.

## Contact

[Your Contact Information]
