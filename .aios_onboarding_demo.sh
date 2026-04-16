#!/bin/bash
# Quick demo of the onboarding system

echo "=========================================="
echo "AI-OS Onboarding Demo"
echo "=========================================="
echo ""
echo "This will demonstrate the onboarding GUI"
echo "Note: Videos require ffmpeg and will be placeholders"
echo ""

# Check if PyQt6 is installed
python3 -c "import PyQt6" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "Installing PyQt6..."
    pip install PyQt6 PyQt6-Multimedia
fi

# Create demo videos if ffmpeg is available
if command -v ffmpeg &> /dev/null; then
    echo "Generating demo videos..."
    python3 tools/generate_demo_videos.py
else
    echo "Note: ffmpeg not found - videos will be placeholders"
fi

echo ""
echo "Launching onboarding wizard..."
python3 run_onboarding.py
