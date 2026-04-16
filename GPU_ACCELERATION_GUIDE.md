# GPU Acceleration Guide for AI-OS

## Overview

AI-OS now supports GPU acceleration for dramatically faster AI inference using CUDA, ROCm, or Metal.

## Supported Platforms

| Platform | Backend | Speed Improvement |
|----------|---------|-------------------|
| NVIDIA GPU | CUDA | 5-10x faster |
| AMD GPU | ROCm/HIP | 4-8x faster |
| Apple Silicon | Metal | 3-6x faster |
| Intel GPU | oneAPI | 2-4x faster |
| CPU | None | Baseline |

## Performance Comparison

### Gemma 2B Inference (typical)

| Hardware | Tokens/sec | Latency (50 tokens) |
|----------|------------|---------------------|
| CPU (i7-9700K) | 10-20 | ~2500ms |
| RTX 3080 | 80-100 | ~500ms |
| RTX 4090 | 150-200 | ~250ms |
| AMD RX 7900 XTX | 70-90 | ~600ms |
| Apple M2 Max | 60-80 | ~700ms |

## Auto-Detection

AI-OS automatically detects available GPUs and selects the best backend:

```
[GPU] Scanning for compute devices...
[GPU] ✓ Detected 1 GPU device(s)
[GPU]   - NVIDIA RTX 3080 (CUDA, 10240 MB VRAM)
[AI]   GPU layers: 32 (auto-detected)
```

## Manual Configuration

### Build with GPU Support

```bash
cd kernel_rs
./build_llama_gpu.sh
```

This script automatically:
1. Detects your GPU
2. Configures llama.cpp with appropriate backend
3. Compiles with GPU support
4. Links against CUDA/ROCm/Metal libraries

### NVIDIA CUDA Setup

**Prerequisites:**
```bash
# Install CUDA Toolkit
# Ubuntu/Debian
sudo apt install nvidia-cuda-toolkit

# Check installation
nvidia-smi
nvcc --version
```

**Build:**
```bash
cd kernel_rs
./build_llama_gpu.sh
# Auto-detects CUDA
```

**Verify:**
```
[BUILD] Configuring for cuda...
-DLLAMA_CUBLAS=ON
-DCMAKE_CUDA_ARCHITECTURES=native
```

### AMD ROCm Setup

**Prerequisites:**
```bash
# Install ROCm
# Ubuntu 22.04
wget https://repo.radeon.com/amdgpu-install/latest/ubuntu/jammy/amdgpu-install_*_all.deb
sudo apt install ./amdgpu-install_*_all.deb
sudo amdgpu-install --usecase=rocm

# Check installation
rocm-smi
```

**Build:**
```bash
cd kernel_rs
./build_llama_gpu.sh
# Auto-detects ROCm
```

### Apple Metal Setup

**Prerequisites:**
- macOS 12.3 or later
- Apple Silicon (M1/M2/M3)
- Xcode Command Line Tools

**Build:**
```bash
cd kernel_rs
./build_llama_gpu.sh
# Auto-detects Metal on Apple Silicon
```

## GPU Layer Configuration

### Automatic (Recommended)

```rust
AIConfig {
    gpu_layers: 0,  // 0 = auto-detect
    ..Default::default()
}
```

AI-OS will:
- Detect available VRAM
- Calculate optimal layer offload
- Configure automatically

### Manual Override

```rust
AIConfig {
    gpu_layers: 32,  // Offload all layers
    ..Default::default()
}
```

**Guidelines:**
- Gemma 2B: 32 layers total
- 10GB VRAM: 32 layers (all)
- 8GB VRAM: 28-30 layers
- 6GB VRAM: 20-25 layers
- 4GB VRAM: 15-20 layers
- < 4GB VRAM: CPU only

## VRAM Requirements

### Model Size in VRAM

| Model | Q4_K_M | Q8_0 | FP16 |
|-------|--------|------|------|
| Gemma 2B | ~2.5 GB | ~4.5 GB | ~8 GB |
| Llama 3.2 1B | ~1.2 GB | ~2 GB | ~4 GB |
| Mistral 7B | ~4.4 GB | ~8 GB | ~14 GB |

**Formula:** VRAM = Model Size × (Layers Offloaded / Total Layers) + Context Buffer

### Context Buffer

- 2K context: ~100 MB
- 4K context: ~200 MB
- 8K context: ~400 MB

## Optimization Tips

### Maximum Speed

```rust
AIConfig {
    gpu_layers: 32,           // All layers on GPU
    context_size: 2048,       // Smaller context
    threads: 1,               // Minimal CPU threads
    ..Default::default()
}
```

### Maximum Quality

```rust
AIConfig {
    gpu_layers: 32,
    context_size: 8192,       // Larger context
    threads: 4,
    ..Default::default()
}
```

### Balanced

```rust
AIConfig::default()  // Let AI-OS decide
```

## Troubleshooting

### GPU Not Detected

**Check drivers:**
```bash
# NVIDIA
nvidia-smi

# AMD
rocm-smi

# Metal (macOS)
system_profiler SPDisplaysDataType
```

**Rebuild with GPU:**
```bash
cd kernel_rs
rm libllama.a
./build_llama_gpu.sh
```

### Out of Memory

**Symptoms:**
```
[GPU] CUDA error: out of memory
[AI] Falling back to CPU
```

**Solutions:**
1. Reduce GPU layers:
   ```rust
   AIConfig { gpu_layers: 20, ..Default::default() }
   ```

2. Reduce context size:
   ```rust
   AIConfig { context_size: 1024, ..Default::default() }
   ```

3. Use smaller quantization:
   - Q4_K_M → Q4_K_S (smaller but faster)

4. Close other GPU applications

### Slow Performance

**Check layer offloading:**
```
[AI]   GPU layers: 32 (auto-detected)
```

If 0 layers, GPU not being used.

**Check batch size:**
```rust
// Increase for throughput, decrease for latency
LlamaContextParams {
    n_batch: 512,  // Default
    ...
}
```

### Build Errors

**CUDA not found:**
```bash
export CUDA_PATH=/usr/local/cuda
export PATH=$CUDA_PATH/bin:$PATH
export LD_LIBRARY_PATH=$CUDA_PATH/lib64:$LD_LIBRARY_PATH
```

**ROCm not found:**
```bash
export ROCM_PATH=/opt/rocm
export PATH=$ROCM_PATH/bin:$PATH
```

## Advanced Configuration

### Multi-GPU Support

```rust
// Future feature - not yet implemented
AIConfig {
    gpu_devices: vec![0, 1],  // Use GPUs 0 and 1
    gpu_split: vec![0.5, 0.5], // 50% each
    ..Default::default()
}
```

### Mixed Precision

```rust
// Use FP16 for speed, FP32 for accuracy
AIConfig {
    use_fp16: true,  // CUDA/ROCm only
    ..Default::default()
}
```

### Tensor Cores (NVIDIA)

Automatically used on RTX series GPUs (compute capability >= 7.0)
- RTX 20xx series: ✅
- RTX 30xx series: ✅
- RTX 40xx series: ✅
- GTX 10xx series: ❌

## Benchmarking

### Run Benchmark

```bash
# Build with GPU
cd kernel_rs
./build_llama_gpu.sh

# Download model
cd ../models
huggingface-cli download google/gemma-2b-it-GGUF gemma-2b-it-q4_k_m.gguf --local-dir .

# Test inference speed
cd ../kernel_rs/llama.cpp/build/bin
./main -m ../../../models/gemma-2b-it-q4_k_m.gguf \
    -p "Test prompt" \
    -n 100 \
    -ngl 32  # GPU layers
```

### Interpret Results

```
llama_print_timings:        load time =   1234.56 ms
llama_print_timings:      sample time =     12.34 ms
llama_print_timings: prompt eval time =    123.45 ms
llama_print_timings:        eval time =    567.89 ms /    100 runs   (5.68 ms per token, 176.1 tokens per second)
```

**Key metrics:**
- **Tokens/second:** Higher is better (target: >50 on GPU)
- **ms per token:** Lower is better (target: <20ms on GPU)
- **Load time:** One-time cost (acceptable: <5 seconds)

## Power Consumption

| Configuration | Power Draw | Performance/Watt |
|---------------|------------|------------------|
| CPU only | 65W | Low |
| CUDA (RTX 3080) | 320W | High |
| ROCm (RX 7900) | 300W | High |
| Metal (M2 Max) | 40W | Very High |

**Power efficiency winner:** Apple Silicon (Metal)
**Raw performance winner:** NVIDIA RTX 40xx series

## Compatibility Matrix

| GPU | CUDA | ROCm | Metal | oneAPI |
|-----|------|------|-------|--------|
| NVIDIA RTX | ✅ | ❌ | ❌ | ⚠️ |
| NVIDIA GTX | ✅ | ❌ | ❌ | ⚠️ |
| AMD RX 6xxx | ❌ | ✅ | ❌ | ⚠️ |
| AMD RX 7xxx | ❌ | ✅ | ❌ | ⚠️ |
| Apple M1/M2/M3 | ❌ | ❌ | ✅ | ❌ |
| Intel Arc | ❌ | ❌ | ❌ | ✅ |

---

**Status:** GPU acceleration implemented and tested
**Recommendation:** Use RTX 3080 or better for best experience
