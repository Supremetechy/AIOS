#!/usr/bin/env python3
"""
AI-OS Demo Script
Demonstrates the capabilities of AI-OS on the current system
"""

from kernel import HardwareDetector, BootLoader, BootConfig

def main():
    print("="*80)
    print("AI-OS DEMONSTRATION")
    print("="*80)
    print()
    
    # 1. Hardware Detection Demo
    print("1. HARDWARE DETECTION")
    print("-" * 80)
    detector = HardwareDetector()
    specs = detector.detect_all()
    
    print(f"\n✓ Detected System: {specs.os_type} {specs.architecture}")
    print(f"✓ Hostname: {specs.hostname}")
    print(f"✓ Kernel: {specs.kernel_version}")
    
    print(f"\n✓ Found {len(specs.processors)} processor(s):")
    for proc in specs.processors:
        print(f"   • {proc.processor_type.value.upper()}: {proc.vendor.value.upper()} - {proc.model}")
        if proc.cores > 0:
            print(f"     Cores: {proc.cores}, Threads: {proc.threads}")
        if proc.memory_gb:
            print(f"     VRAM: {proc.memory_gb:.2f} GB")
        if proc.capabilities:
            caps = ', '.join(proc.capabilities[:5])
            print(f"     Capabilities: {caps}")
    
    print(f"\n✓ System Memory: {specs.memory.total_gb:.2f} GB")
    print(f"   Available: {specs.memory.available_gb:.2f} GB")
    
    print(f"\n✓ Storage: {len(specs.storage_devices)} device(s)")
    for device in specs.storage_devices[:3]:
        print(f"   • {device.mount_point}: {device.total_gb:.1f} GB ({device.filesystem_type})")
    
    print(f"\n✓ Network: {len(specs.network_interfaces)} interface(s)")
    
    # 2. AI Capabilities Summary
    print("\n\n2. AI WORKLOAD CAPABILITIES")
    print("-" * 80)
    
    has_cpu = any(p.processor_type.value == 'cpu' for p in specs.processors)
    has_gpu = any(p.processor_type.value == 'gpu' for p in specs.processors)
    has_tpu = any(p.processor_type.value == 'tpu' for p in specs.processors)
    
    print("\nSupported AI Frameworks:")
    if has_cpu:
        print("  ✓ PyTorch (CPU)")
        print("  ✓ TensorFlow (CPU)")
        print("  ✓ JAX (CPU)")
    
    if has_gpu:
        for proc in specs.processors:
            if proc.processor_type.value == 'gpu':
                if proc.vendor.value == 'nvidia':
                    print("  ✓ PyTorch (CUDA)")
                    print("  ✓ TensorFlow (CUDA)")
                    print("  ✓ JAX (CUDA)")
                    print("  ✓ RAPIDS")
                elif proc.vendor.value == 'amd':
                    print("  ✓ PyTorch (ROCm)")
                    print("  ✓ TensorFlow (ROCm)")
                elif proc.vendor.value == 'apple':
                    print("  ✓ PyTorch (MPS)")
                    print("  ✓ TensorFlow (Metal)")
                    print("  ✓ Core ML")
    
    if has_tpu:
        print("  ✓ TensorFlow (TPU)")
        print("  ✓ JAX (TPU)")
    
    print("\nRecommended Workloads:")
    total_memory = specs.memory.total_gb
    total_vram = sum(p.memory_gb for p in specs.processors if p.memory_gb)
    
    if total_vram >= 24:
        print("  ✓ Large Language Models (7B-13B parameters)")
        print("  ✓ Image Generation (Stable Diffusion XL)")
        print("  ✓ Video Processing")
    elif total_vram >= 8:
        print("  ✓ Medium Language Models (3B-7B parameters)")
        print("  ✓ Image Generation (Stable Diffusion)")
        print("  ✓ Object Detection")
    elif has_gpu:
        print("  ✓ Small Language Models (<3B parameters)")
        print("  ✓ Image Classification")
        print("  ✓ Basic Computer Vision")
    
    if total_memory >= 64:
        print("  ✓ Large Dataset Processing")
        print("  ✓ Distributed Training")
    elif total_memory >= 16:
        print("  ✓ Standard Model Training")
        print("  ✓ Inference Serving")
    
    # 3. System Export
    print("\n\n3. SYSTEM SPECIFICATIONS EXPORT")
    print("-" * 80)
    
    json_output = detector.to_json(specs)
    with open('system_specs.json', 'w') as f:
        f.write(json_output)
    print("✓ System specifications saved to: system_specs.json")
    
    # 4. Boot Test
    print("\n\n4. BOOT SYSTEM TEST")
    print("-" * 80)
    print("Testing boot sequence...\n")
    
    config = BootConfig(verbose=False, skip_hardware_check=True)
    bootloader = BootLoader(config)
    bootloader.boot()
    
    print("\n\n" + "="*80)
    print("DEMO COMPLETE")
    print("="*80)
    print("\nYour AI-OS is ready! Run 'python3 aios_kernel.py' to start the full kernel.")
    print()

if __name__ == "__main__":
    main()
