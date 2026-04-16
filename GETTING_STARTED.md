# Getting Started with AI-OS

## Quick Start Guide

### 1. Run the Demo
```bash
python3 demo.py
```
This will:
- Detect all hardware (CPU, GPU, TPU, NPU)
- Show AI framework compatibility
- Recommend workloads based on your hardware
- Export system specs to JSON
- Run a quick boot test

### 2. Start the Full Kernel
```bash
python3 aios_kernel.py
```

Once running, you'll see an interactive shell:
```
aios> help        # Show all commands
aios> status      # Current system status
aios> monitor     # Real-time monitoring
aios> hardware    # Hardware info
aios> network     # Network config
aios> filesystem  # Filesystem info
aios> specs       # Detailed specs
aios> exit        # Shutdown
```

### 3. Quick Monitoring
```bash
python3 aios_kernel.py --monitor-only
```
Shows instant snapshot of system resources.

## What Your System Supports

Based on the detected hardware, AI-OS will automatically:

### If you have NVIDIA GPU:
- Load CUDA drivers
- Enable PyTorch (CUDA), TensorFlow (CUDA), JAX (CUDA)
- Support for RAPIDS, cuDNN, TensorRT
- GPU memory tracking and temperature monitoring

### If you have AMD GPU:
- Load ROCm drivers
- Enable PyTorch (ROCm), TensorFlow (ROCm)
- HIP support

### If you have Apple Silicon:
- Load Metal drivers
- Enable PyTorch (MPS), TensorFlow (Metal)
- Core ML and Neural Engine support

### If you have Google TPU:
- Load TPU runtime
- Enable TensorFlow (TPU), JAX (TPU), PyTorch/XLA

## Directory Structure

```
ai-os/
├── aios_kernel.py              # Main kernel (run this!)
├── demo.py                     # Demo script
├── kernel/                     # Core OS components
│   ├── hardware_detection.py  # Hardware detection
│   ├── boot.py                 # Boot system
│   ├── filesystem.py           # Filesystem management
│   ├── network.py              # Network stack
│   └── system_monitor.py       # Real-time monitoring
├── README_AIOS.md              # Full documentation
└── GETTING_STARTED.md          # This file
```

## Common Use Cases

### 1. Check if you have GPU acceleration
```bash
python3 demo.py
# Look for "GPU:" in the output
```

### 2. Monitor GPU usage during training
```bash
python3 aios_kernel.py
aios> monitor
# Shows GPU memory, temperature, power usage
```

### 3. Check available disk space for models
```bash
python3 aios_kernel.py
aios> filesystem
# Shows all mount points and usage
```

### 4. Verify network for distributed training
```bash
python3 aios_kernel.py
aios> network
# Shows interfaces and connectivity
```

## Example Outputs

### Hardware Detection on NVIDIA System:
```
✓ Found 2 processor(s):
   • CPU: INTEL - Intel(R) Core(TM) i9-9900K
     Cores: 8, Threads: 16
   • GPU: NVIDIA - NVIDIA GeForce RTX 3090
     VRAM: 24.00 GB
     Capabilities: CUDA, cuDNN, TensorRT

Supported AI Frameworks:
  ✓ PyTorch (CUDA)
  ✓ TensorFlow (CUDA)
  ✓ JAX (CUDA)
  ✓ RAPIDS

Recommended Workloads:
  ✓ Large Language Models (7B-13B parameters)
  ✓ Image Generation (Stable Diffusion XL)
  ✓ Video Processing
```

### Hardware Detection on Apple Silicon:
```
✓ Found 1 processor(s):
   • GPU: APPLE - Apple M1 Pro GPU
     Capabilities: Metal, Neural Engine

Supported AI Frameworks:
  ✓ PyTorch (MPS)
  ✓ TensorFlow (Metal)
  ✓ Core ML

Recommended Workloads:
  ✓ Small Language Models (<3B parameters)
  ✓ Image Classification
  ✓ Basic Computer Vision
```

## Integration with AI Frameworks

### PyTorch Example
```python
import torch
from kernel import HardwareDetector

# Detect hardware
detector = HardwareDetector()
specs = detector.detect_all()

# Choose device based on detected hardware
if any(p.processor_type.value == 'gpu' and p.vendor.value == 'nvidia' 
       for p in specs.processors):
    device = torch.device('cuda')
elif any(p.processor_type.value == 'gpu' and p.vendor.value == 'apple' 
         for p in specs.processors):
    device = torch.device('mps')
else:
    device = torch.device('cpu')

print(f"Using device: {device}")
```

### Check Available VRAM
```python
from kernel import HardwareDetector

detector = HardwareDetector()
specs = detector.detect_all()

for proc in specs.processors:
    if proc.processor_type.value == 'gpu' and proc.memory_gb:
        print(f"GPU: {proc.model}")
        print(f"VRAM: {proc.memory_gb:.2f} GB")
        
        # Estimate model size you can load
        # Rule of thumb: use 80% of VRAM
        max_model_gb = proc.memory_gb * 0.8
        print(f"Can load models up to ~{max_model_gb:.1f} GB")
```

### Monitor Training
```python
from kernel import SystemMonitor

monitor = SystemMonitor()

# During training loop
snapshot = monitor.get_snapshot()
for gpu in snapshot.gpu_stats:
    print(f"GPU {gpu.gpu_id}: {gpu.usage_percent:.1f}% usage")
    print(f"  Memory: {gpu.memory_percent:.1f}%")
    print(f"  Temp: {gpu.temperature_celsius:.1f}°C")
```

## Troubleshooting

### "No GPU detected" but you have one
1. Make sure GPU drivers are installed:
   - NVIDIA: `nvidia-smi` should work
   - AMD: `rocm-smi` should work
2. Check if GPU is enabled in BIOS
3. Verify GPU is properly seated

### "Permission denied" errors
```bash
# Make script executable
chmod +x aios_kernel.py demo.py
```

### Network interfaces not showing
- Run with verbose mode: `python3 aios_kernel.py --verbose`
- Check: `ip addr` (Linux) or `ifconfig` (macOS)

### Memory/CPU stats showing 0
- On macOS: Some stats require different APIs
- This is expected, GPU/CPU detection still works

## Next Steps

1. **Run the demo**: `python3 demo.py`
2. **Check your specs**: Look at `system_specs.json`
3. **Start the kernel**: `python3 aios_kernel.py`
4. **Explore commands**: Type `help` in the shell

## Advanced Usage

### Boot Options
```bash
# Safe mode (skip some checks)
python3 aios_kernel.py --safe-mode

# Verbose output
python3 aios_kernel.py --verbose

# No network
python3 aios_kernel.py --no-network

# No filesystem auto-mount
python3 aios_kernel.py --no-fs
```

### Programmatic Access
```python
from kernel import HardwareDetector, BootLoader, SystemMonitor

# Detect hardware
detector = HardwareDetector()
specs = detector.detect_all()

# Boot system
from kernel import BootConfig
config = BootConfig(verbose=True)
bootloader = BootLoader(config)
bootloader.boot()

# Monitor resources
monitor = SystemMonitor()
snapshot = monitor.get_snapshot()
print(f"CPU: {snapshot.cpu_stats.usage_percent:.1f}%")
```

## Support

For issues or questions, refer to README_AIOS.md for full documentation.
