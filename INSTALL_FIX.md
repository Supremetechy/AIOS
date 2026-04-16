# Installation Fix - PyQt6-Multimedia Issue

## Problem
`PyQt6-Multimedia` may not be available in some Python environments or repositories.

## Solution
We've created a **fallback version** that works with PyQt6 only (no multimedia dependency).

## Quick Fix

### Option 1: Install PyQt6 Only (Recommended)
```bash
pip install PyQt6
python run_onboarding.py
```

The launcher will automatically detect the missing multimedia package and use the fallback version with video placeholders.

### Option 2: Try Installing Full Package
```bash
pip install PyQt6
pip install PyQt6-Qt6
```

### Option 3: Use Conda (If Available)
```bash
conda install -c conda-forge pyqt
```

## What's Different?

### Full Version (kernel/onboarding_gui.py)
- Uses PyQt6-Multimedia
- Plays actual MP4 videos
- Full video controls

### Fallback Version (kernel/onboarding_gui_fallback.py)
- Uses PyQt6 only
- Shows styled video placeholders
- All functionality works, just no actual video playback
- Automatically used when multimedia is not available

## Current Status

✅ **You can use the onboarding wizard now!**

The fallback version provides:
- Complete 5-step wizard
- Hardware detection
- GPU configuration
- System settings
- All features except actual video playback

## Testing

```bash
# Test what you have
python test_onboarding.py

# Run the wizard
python run_onboarding.py
```

## Adding Videos Later

When you want to add videos:

1. Install multimedia support (if available):
   ```bash
   pip install PyQt6-Qt6
   ```

2. Or use the fallback version with placeholders that show:
   - Video title
   - Video icon
   - Information about what the video would show

## Files Updated

- `requirements_gui.txt` - Made multimedia optional
- `run_onboarding.py` - Auto-detects and uses fallback
- `kernel/onboarding_gui_fallback.py` - New fallback version

## Both Versions Include

✅ Full onboarding wizard
✅ Hardware detection
✅ GPU auto-configuration  
✅ System configuration
✅ All navigation features
✅ Configuration persistence
✅ AI-OS launch integration

The **only difference** is video playback vs. placeholder display.
