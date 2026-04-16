# AI-OS Quick Start Guide - On-Device AI Edition

Get your AI-OS kernel running with native on-device AI in 4 steps!

## 🚀 Quick Setup (30 minutes)

### Prerequisites

```bash
# Ubuntu/Debian
sudo apt install build-essential cmake git cargo grub-pc-bin xorriso

# macOS
brew install cmake git rust grub xorriso

# Fedora/RHEL
sudo dnf install gcc-c++ cmake git cargo grub2-tools xorriso
```

### Step 1: Clone and Build llama.cpp (5 minutes)

```bash
cd kernel_rs
./build_llama.sh
```

This will:
- Clone llama.cpp repository
- Build as static library
- Link with AI-OS kernel

**Output:** `kernel_rs/libllama.a` (~50MB)

### Step 2: Download AI Model (10-15 minutes)

**Option A: Using Hugging Face CLI (Recommended)**

```bash
pip install huggingface-hub

cd models
huggingface-cli download google/gemma-2b-it-GGUF \
    gemma-2b-it-q4_k_m.gguf \
    --local-dir .
```

**Option B: Direct Download**

Visit: https://huggingface.co/google/gemma-2b-it-GGUF

Download: `gemma-2b-it-q4_k_m.gguf` (2.5GB)

Save to: `models/gemma-2b-it-q4_k_m.gguf`

**Alternative Models** (see `models/README.md` for more):
- Smaller: `llama-3.2-1b-instruct-q4_k_m.gguf` (800MB)
- Larger: `mistral-7b-instruct-v0.2.q4_k_m.gguf` (4.4GB)

### Step 3: Build ISO with AI (5 minutes)

```bash
./build_iso_with_ai.sh
```

This creates: `aios-ai-YYYYMMDD.iso` (~3GB)

The ISO includes:
- AI-OS kernel with AI subsystem
- llama.cpp inference engine
- Gemma 2B model (embedded)
- Boot configuration

### Step 4: Test in QEMU (Immediate)

```bash
./run_qemu.sh aios-ai-YYYYMMDD.iso
```

**Expected Boot Sequence:**

```
[KERNEL] Initializing AI subsystem...
[AI]   Model: /models/gemma-2b-it-q4_k_m.gguf
[AI]   Context size: 2048 tokens
[AI]   Threads: 4
[AI]   GPU layers: 32
[LLAMA] Initializing llama.cpp backend...
[LLAMA] ✓ Backend initialized
[LLAMA] Loading model: /models/gemma-2b-it-q4_k_m.gguf
[LLAMA] ✓ Model loaded
[LLAMA] ✓ Context created
[AI] ✓ AI subsystem initialized

╔════════════════════════════════════════════════════════════════╗
║              AI-OS Interactive Assistant                      ║
║            Native On-Device AI (Gemma 2B)                      ║
╚════════════════════════════════════════════════════════════════╝

[AI] Initializing conversation...