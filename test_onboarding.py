#!/usr/bin/env python3
"""
Test script for AI-OS Onboarding System
Verifies all components are working correctly
"""

import sys
import os
from pathlib import Path

def test_imports():
    """Test that all required modules can be imported"""
    print("Testing imports...")
    
    tests = []
    
    # Test PyQt6
    try:
        from PyQt6.QtWidgets import QApplication
        from PyQt6.QtMultimedia import QMediaPlayer
        tests.append(("PyQt6", True, ""))
    except ImportError as e:
        tests.append(("PyQt6", False, str(e)))
    
    # Test onboarding module
    try:
        from kernel.onboarding_gui import (
            OnboardingConfig, VideoPlayer, AIAssistantVideos,
            OnboardingWizard, WelcomeStep, HardwareDetectionStep
        )
        tests.append(("Onboarding GUI", True, ""))
    except ImportError as e:
        tests.append(("Onboarding GUI", False, str(e)))
    
    # Test hardware detection
    try:
        from kernel.hardware_detection import HardwareDetector
        tests.append(("Hardware Detection", True, ""))
    except ImportError as e:
        tests.append(("Hardware Detection", False, str(e)))
    
    # Print results
    all_passed = True
    for name, passed, error in tests:
        status = "✓" if passed else "✗"
        print(f"  {status} {name}")
        if not passed:
            print(f"    Error: {error}")
            all_passed = False
    
    return all_passed


def test_configuration():
    """Test configuration management"""
    print("\nTesting configuration...")
    
    try:
        from kernel.onboarding_gui import OnboardingConfig
        
        config = OnboardingConfig()
        
        # Test config structure
        assert isinstance(config.config, dict), "Config should be a dictionary"
        assert "onboarding_complete" in config.config, "Missing onboarding_complete key"
        
        # Test mark step complete
        config.mark_step_complete(0)
        assert config.is_step_complete(0), "Step should be marked complete"
        
        print("  ✓ Configuration management works")
        return True
        
    except Exception as e:
        print(f"  ✗ Configuration test failed: {e}")
        return False


def test_video_resources():
    """Test video resource management"""
    print("\nTesting video resources...")
    
    try:
        from kernel.onboarding_gui import AIAssistantVideos
        
        # Test video metadata
        videos = AIAssistantVideos.VIDEOS
        assert len(videos) == 6, f"Expected 6 videos, found {len(videos)}"
        
        required_videos = ["welcome", "hardware_setup", "gpu_config", 
                          "network_setup", "security", "complete"]
        
        for video_key in required_videos:
            info = AIAssistantVideos.get_video_info(video_key)
            assert info is not None, f"Video info missing for {video_key}"
            assert "file" in info, f"File path missing for {video_key}"
            assert "title" in info, f"Title missing for {video_key}"
        
        print("  ✓ Video resource management works")
        print(f"  ℹ {len(videos)} videos configured")
        return True
        
    except Exception as e:
        print(f"  ✗ Video resources test failed: {e}")
        return False


def test_file_structure():
    """Test that all required files exist"""
    print("\nTesting file structure...")
    
    required_files = [
        "kernel/onboarding_gui.py",
        "run_onboarding.py",
        "requirements_gui.txt",
        "ONBOARDING_GUIDE.md",
        "QUICKSTART_ONBOARDING.md",
        "ONBOARDING_SUMMARY.md",
        "tools/generate_demo_videos.py",
        "assets/onboarding_videos/README.md"
    ]
    
    all_exist = True
    for file_path in required_files:
        exists = Path(file_path).exists()
        status = "✓" if exists else "✗"
        print(f"  {status} {file_path}")
        if not exists:
            all_exist = False
    
    return all_exist


def test_video_directory():
    """Test video directory setup"""
    print("\nTesting video directory...")
    
    videos_dir = Path("assets/onboarding_videos")
    
    if not videos_dir.exists():
        print(f"  ℹ Video directory doesn't exist yet (will be created on first run)")
        return True
    
    print(f"  ✓ Video directory exists: {videos_dir}")
    
    # Check for video files
    video_files = list(videos_dir.glob("*.mp4"))
    if video_files:
        print(f"  ✓ Found {len(video_files)} video file(s):")
        for vf in video_files:
            size_mb = vf.stat().st_size / (1024 * 1024)
            print(f"    - {vf.name} ({size_mb:.2f} MB)")
    else:
        print(f"  ℹ No video files yet (generate with tools/generate_demo_videos.py)")
    
    return True


def main():
    """Run all tests"""
    print("=" * 60)
    print("AI-OS Onboarding System - Test Suite")
    print("=" * 60)
    print()
    
    results = []
    
    # Run tests
    results.append(("Imports", test_imports()))
    results.append(("Configuration", test_configuration()))
    results.append(("Video Resources", test_video_resources()))
    results.append(("File Structure", test_file_structure()))
    results.append(("Video Directory", test_video_directory()))
    
    # Summary
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)
    
    passed = sum(1 for _, result in results if result)
    total = len(results)
    
    for name, result in results:
        status = "PASS" if result else "FAIL"
        print(f"  {status}: {name}")
    
    print()
    print(f"Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("\n✅ All tests passed! Onboarding system is ready.")
        print("\nNext steps:")
        print("  1. Install dependencies: pip install -r requirements_gui.txt")
        print("  2. Generate demo videos: python tools/generate_demo_videos.py")
        print("  3. Launch wizard: python run_onboarding.py")
        return 0
    else:
        print("\n❌ Some tests failed. Please review errors above.")
        print("\nTo fix:")
        print("  1. Install PyQt6: pip install PyQt6 PyQt6-Multimedia")
        print("  2. Ensure all kernel modules are present")
        print("  3. Run setup: ./setup_onboarding.sh")
        return 1


if __name__ == "__main__":
    sys.exit(main())
