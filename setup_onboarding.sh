#!/bin/bash
# AI-OS Onboarding Setup Script
# Sets up the onboarding system and generates demo videos

set -e

echo "=========================================="
echo "AI-OS Onboarding Setup"
echo "=========================================="
echo ""

# Check Python version
echo "Checking Python version..."
python3 --version || { echo "Error: Python 3 not found"; exit 1; }
echo "✓ Python 3 found"
echo ""

# Install Python dependencies
echo "Installing Python dependencies..."
pip install -r requirements_gui.txt
echo "✓ Dependencies installed"
echo ""

# Create directories
echo "Creating directories..."
mkdir -p assets/onboarding_videos
mkdir -p tools
mkdir -p ~/.aios
echo "✓ Directories created"
echo ""

# Check for ffmpeg (optional)
if command -v ffmpeg &> /dev/null; then
    echo "✓ ffmpeg found - demo videos can be generated"
    echo ""
    
    # Ask if user wants to generate demo videos
    read -p "Generate demo placeholder videos? (y/n) " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Generating demo videos..."
        python tools/generate_demo_videos.py
        echo ""
    fi
else
    echo "⚠ ffmpeg not found - demo videos cannot be generated"
    echo "  Install ffmpeg to generate placeholder videos:"
    echo "    macOS:  brew install ffmpeg"
    echo "    Linux:  sudo apt-get install ffmpeg"
    echo ""
fi

# Make scripts executable
echo "Setting permissions..."
chmod +x run_onboarding.py
chmod +x tools/generate_demo_videos.py
echo "✓ Permissions set"
echo ""

# Summary
echo "=========================================="
echo "Setup Complete!"
echo "=========================================="
echo ""
echo "To launch the onboarding wizard:"
echo "  ./run_onboarding.py"
echo ""
echo "Or:"
echo "  python3 run_onboarding.py"
echo ""
echo "For more information, see ONBOARDING_GUIDE.md"
echo ""
