#!/usr/bin/env python3
"""
Generate demo placeholder videos for onboarding
Uses ffmpeg to create simple placeholder videos for testing
"""

import subprocess
import sys
from pathlib import Path

VIDEOS_DIR = Path("assets/onboarding_videos")

# Video specifications
VIDEOS = {
    "welcome": {
        "title": "Welcome to AI-OS",
        "subtitle": "Your AI-Powered Operating System",
        "color": "0x2c3e50",
        "duration": 30
    },
    "hardware_setup": {
        "title": "Hardware Detection & Setup",
        "subtitle": "Optimizing your system for AI",
        "color": "0x27ae60",
        "duration": 30
    },
    "gpu_configuration": {
        "title": "GPU Configuration",
        "subtitle": "Accelerating AI workloads",
        "color": "0x2980b9",
        "duration": 30
    },
    "network_setup": {
        "title": "Network Configuration",
        "subtitle": "Distributed training ready",
        "color": "0x8e44ad",
        "duration": 30
    },
    "security_overview": {
        "title": "Security & Privacy",
        "subtitle": "Keeping your AI secure",
        "color": "0xe74c3c",
        "duration": 30
    },
    "setup_complete": {
        "title": "Setup Complete!",
        "subtitle": "Ready to launch AI-OS",
        "color": "0x16a085",
        "duration": 30
    }
}


def check_ffmpeg():
    """Check if ffmpeg is installed"""
    try:
        subprocess.run(["ffmpeg", "-version"], capture_output=True, check=True)
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False


def find_font():
    """Find an available font on the system"""
    possible_fonts = [
        "/Library/Fonts/Arial Unicode.ttf",  # macOS
        "/System/Library/Fonts/Geneva.ttf",  # macOS fallback
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",  # Linux
        "/usr/share/fonts/TTF/DejaVuSans.ttf",  # Linux (Arch)
        "C:\\Windows\\Fonts\\arial.ttf",  # Windows
    ]
    
    for font in possible_fonts:
        if Path(font).exists():
            return font
    
    # If no font found, try without fontfile parameter (uses system default)
    return None


def generate_video(name: str, config: dict):
    """Generate a placeholder video using ffmpeg"""
    output_path = VIDEOS_DIR / f"{name}.mp4"
    
    title = config["title"]
    subtitle = config["subtitle"]
    color = config["color"]
    duration = config["duration"]
    
    # Find available font
    font = find_font()
    
    # Try to create video with text first
    if font:
        drawtext = (
            f"drawtext=text='{title}':fontsize=48:fontcolor=white:x=(w-text_w)/2:y=(h-text_h)/2-40:fontfile={font},"
            f"drawtext=text='{subtitle}':fontsize=24:fontcolor=white:x=(w-text_w)/2:y=(h-text_h)/2+40:fontfile={font}"
        )
    else:
        # Fallback without fontfile
        drawtext = (
            f"drawtext=text='{title}':fontsize=48:fontcolor=white:x=(w-text_w)/2:y=(h-text_h)/2-40,"
            f"drawtext=text='{subtitle}':fontsize=24:fontcolor=white:x=(w-text_w)/2:y=(h-text_h)/2+40"
        )
    
    # Try FFmpeg command with text overlay
    cmd_with_text = [
        "ffmpeg", "-y",
        "-f", "lavfi",
        "-i", f"color=c={color}:s=1280x720:d={duration}",
        "-vf", drawtext,
        "-c:v", "libx264",
        "-pix_fmt", "yuv420p",
        "-preset", "fast",
        str(output_path)
    ]
    
    # Fallback command without text (if drawtext filter not available)
    cmd_simple = [
        "ffmpeg", "-y",
        "-f", "lavfi",
        "-i", f"color=c={color}:s=1280x720:d={duration}",
        "-c:v", "libx264",
        "-pix_fmt", "yuv420p",
        "-preset", "fast",
        str(output_path)
    ]
    
    print(f"Generating {name}.mp4...")
    try:
        # Try with text first
        result = subprocess.run(cmd_with_text, capture_output=True, text=True)
        if result.returncode != 0 and "drawtext" in result.stderr:
            # Drawtext filter not available, use simple version
            print(f"  Note: Creating simple placeholder (drawtext filter not available)")
            subprocess.run(cmd_simple, check=True, capture_output=True)
        elif result.returncode != 0:
            raise subprocess.CalledProcessError(result.returncode, cmd_with_text, result.stderr)
        
        print(f"✓ Created {output_path}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"✗ Failed to create {name}.mp4: {e}")
        return False


def main():
    """Main entry point"""
    print("=" * 60)
    print("AI-OS Demo Video Generator")
    print("=" * 60)
    print()
    
    # Check for ffmpeg
    if not check_ffmpeg():
        print("❌ ffmpeg is not installed!")
        print("\nInstall ffmpeg:")
        print("  macOS:  brew install ffmpeg")
        print("  Linux:  sudo apt-get install ffmpeg")
        print("  Windows: Download from https://ffmpeg.org/")
        sys.exit(1)
    
    print("✓ ffmpeg found")
    print()
    
    # Create videos directory
    VIDEOS_DIR.mkdir(parents=True, exist_ok=True)
    print(f"✓ Videos directory: {VIDEOS_DIR}")
    print()
    
    # Generate all videos
    success_count = 0
    for name, config in VIDEOS.items():
        if generate_video(name, config):
            success_count += 1
    
    print()
    print("=" * 60)
    print(f"Generated {success_count}/{len(VIDEOS)} videos successfully")
    print("=" * 60)
    print()
    print("Note: These are placeholder videos for testing.")
    print("Replace with professional AI talking head videos for production.")


if __name__ == "__main__":
    main()
