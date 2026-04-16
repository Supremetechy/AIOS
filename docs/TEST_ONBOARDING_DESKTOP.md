# Testing AI-OS Onboarding Wizard on Desktop

## Quick Test Guide

### Prerequisites Check
```bash
# Verify PyQt6 is installed
python3 -c "import PyQt6; print('✓ PyQt6 installed:', PyQt6.__version__)"

# Check what version will be used
python3 run_onboarding.py --help 2>&1 | head -5 || echo "Run without --help flag"
```

### Launch the Wizard
```bash
python3 run_onboarding.py
```

### Expected Behavior

#### 1. Console Output
You should see:
```
============================================================
AI-OS Onboarding Wizard
============================================================

⚠️  PyQt6-Multimedia not available
   Using fallback version (video placeholders only)

✓ Starting onboarding wizard...
```

#### 2. GUI Window Opens
A window should appear with:
- Title: "AI-OS Setup Wizard"
- Size: 900x700 pixels
- Centered on screen

#### 3. Step 1: Welcome Screen
You should see:
- Large "Welcome to AI-OS" title
- Subtitle: "Your AI-Powered Operating System"
- Video placeholder with gradient blue background
- 🎥 icon in the center
- Text: "AI Assistant Video"
- Feature list below
- [Next →] button at bottom

### Step-by-Step Testing

#### Test 1: Welcome Screen
```
✓ Window appears
✓ Styled video placeholder visible
✓ Text is readable
✓ Next button is enabled
✓ Back button is disabled (first step)
✓ Progress bar shows 0%
```

**Action:** Click [Next →]

---

#### Test 2: Hardware Detection
```
✓ Video placeholder changes to "Hardware Detection Guide"
✓ "Detect Hardware" button visible
✓ Status shows: "Click 'Detect Hardware' to scan your system"
✓ Progress bar shows ~20%
✓ Back button is now enabled
```

**Action:** Click [🔍 Detect Hardware]

**Expected:**
- Button becomes disabled
- Progress bar appears (spinning)
- Status changes to "Detecting hardware..."
- After ~2 seconds:
  - Hardware info appears in text box
  - Status shows "✓ Hardware detection complete!"
  - Button re-enables

**Hardware Info Should Show:**
```
=== Detected Hardware ===

CPU: [Your CPU Model]
Cores: [Number]

GPUs:
  1. [Your GPU]
     Memory: [VRAM]

Memory: [RAM]GB

Storage:
  • [Disk]: [Size]GB
```

**Action:** Click [Next →]

---

#### Test 3: GPU Configuration
```
✓ Video placeholder: "GPU Configuration Guide"
✓ GPU Settings group box visible
✓ Checkboxes auto-selected based on detected GPU:
  - ☑ Enable GPU Acceleration
  - ☑ Enable CUDA (if NVIDIA detected)
  - ☑ Enable ROCm (if AMD detected)
  - ☑ Enable Metal (if Apple detected)
✓ Progress bar shows ~40%
```

**Action:** Review settings, click [Next →]

---

#### Test 4: System Configuration
```
✓ "Basic Settings" form visible
✓ Input fields:
  - Your Name (text input)
  - System Name (text input with placeholder)
  - Installation Path (with Browse button)
  - Auto-start checkbox
  - Telemetry checkbox
✓ Progress bar shows ~60%
```

**Action:** 
1. Enter your name: "Test User"
2. Enter system name: "test-ai-system"
3. Leave other settings as default
4. Click [Next →]

**Validation Test:**
- Try clicking [Next →] WITHOUT filling in name
- Should show warning: "Please fill in your name and system name."
- Fill in both fields, then click [Next →]

---

#### Test 5: Completion Screen
```
✓ Title: "🎉 Setup Complete!"
✓ Video placeholder: "Setup Complete"
✓ Congratulations message visible
✓ "Next Steps" group box with HTML list
✓ Progress bar shows 100%
✓ [Next →] button is hidden
✓ [🚀 Launch AI-OS] button is visible
```

**Action:** Click [🚀 Launch AI-OS]

**Expected:**
- Dialog appears: "AI-OS setup is complete!"
- Two options: [Launch Now] [Exit]

**Action:** Click [Exit] (for testing)

**Expected:**
- Window closes
- Configuration saved to ~/.aios/onboarding_config.json

---

### Verify Configuration Saved

```bash
# Check config file exists
cat ~/.aios/onboarding_config.json
```

**Should contain:**
```json
{
  "onboarding_complete": true,
  "current_step": 0,
  "user_name": "Test User",
  "system_name": "test-ai-system",
  "install_path": "/current/path",
  "gpu_enabled": false,
  "auto_start": false,
  "telemetry_enabled": true,
  "completed_steps": [0, 1, 2, 3]
}
```

---

### Navigation Testing

#### Test Back Button
1. Launch wizard: `python3 run_onboarding.py`
2. Click [Next →] to go to step 2
3. Click [← Back] to return to step 1
4. Verify you're back at Welcome screen

#### Test Skip Hardware Detection
1. Go to step 2 (Hardware Detection)
2. WITHOUT clicking "Detect Hardware", click [Next →]
3. Should show dialog: "Skip Hardware Detection?"
4. Click [Yes] to continue
5. GPU config step should work (no auto-detection)

---

### Visual Elements to Verify

#### Video Placeholders
Each should have:
```
✓ Gradient background (blue → dark blue)
✓ 🎥 icon (large, white)
✓ Video title (large, bold, white)
✓ Subtitle text (white, smaller)
✓ Rounded corners (border-radius: 10px)
✓ Border (2px solid)
```

#### Buttons
```
✓ Blue background (#3498db)
✓ White text
✓ Rounded corners
✓ Hover effect (darker blue)
✓ Disabled state (gray)
```

#### Progress Bar
```
✓ Shows percentage text
✓ Blue fill color
✓ Updates with each step
✓ 0% → 20% → 40% → 60% → 80% → 100%
```

---

### Troubleshooting

#### Issue: Window doesn't appear
```bash
# Check if running in headless environment
echo $DISPLAY

# If empty, you're in headless mode (no GUI possible)
# Need to run on desktop system with display
```

#### Issue: Import errors
```bash
# Verify PyQt6
python3 -c "from PyQt6.QtWidgets import QApplication; print('OK')"

# If fails:
pip install PyQt6
```

#### Issue: Hardware detection fails
- Non-critical! You can skip this step
- Click "Yes" when prompted to skip
- Rest of wizard will work fine

#### Issue: Styling looks different
- Normal! Styling may vary by platform
- Windows/macOS/Linux have different themes
- Functionality should be identical

---

### Quick Test Checklist

Run through all steps and check:

- [ ] Window opens and is centered
- [ ] All 5 steps are accessible
- [ ] Video placeholders display correctly
- [ ] Hardware detection runs (or can be skipped)
- [ ] GPU checkboxes auto-select
- [ ] Form validation works
- [ ] Back/Next navigation works
- [ ] Progress bar updates
- [ ] Configuration saves
- [ ] Launch option appears at end

---

### Screenshot Test (Optional)

Take screenshots of each step for documentation:

```bash
# On macOS
screencapture -w step1_welcome.png

# On Linux
gnome-screenshot -w

# On Windows
# Use Snipping Tool
```

---

### Performance Test

Monitor resource usage:

```bash
# While wizard is running
top | grep -i python

# Should use minimal resources:
# CPU: <5% (idle), <20% (during hardware detection)
# Memory: <100 MB
```

---

### Success Criteria

✅ All 5 steps accessible and functional
✅ Navigation works (Back/Next)
✅ Hardware detection completes or can be skipped
✅ Form validation prevents empty submissions
✅ Configuration persists to file
✅ Professional appearance
✅ No crashes or errors

---

## After Testing

### What to Report

1. **Success:** 
   ```
   ✓ Tested on [OS] [version]
   ✓ All steps working
   ✓ Configuration saved successfully
   ```

2. **Issues Found:**
   ```
   ⚠️ Step X had issue: [description]
   ⚠️ Error message: [if any]
   ⚠️ Expected: [behavior]
   ⚠️ Actual: [what happened]
   ```

### Next Steps After Successful Test

1. **Add Professional Videos** (optional)
   - See: assets/onboarding_videos/README.md

2. **Customize Branding** (optional)
   - Edit: kernel/onboarding_gui_fallback.py
   - Change colors, fonts, text

3. **Deploy to Users**
   - Package the application
   - Include documentation
   - Share with team

---

## Remote Testing (If You Can't Run GUI)

If you're on a server/headless system:

1. **Install on local desktop:**
   ```bash
   # Copy files to your desktop/laptop
   scp -r user@server:/path/to/AIOS ~/Desktop/AIOS
   
   # Install PyQt6 locally
   pip install PyQt6
   
   # Run
   python3 run_onboarding.py
   ```

2. **Use X11 forwarding (advanced):**
   ```bash
   ssh -X user@server
   python3 run_onboarding.py
   # GUI will display on your local machine
   ```

3. **Record video for review:**
   ```bash
   # On desktop, record the wizard
   # Share video for review
   ```

---

## Ready to Test?

Run this command and report back what you see:

```bash
python3 run_onboarding.py
```

Tell me:
1. Did the window open?
2. What step are you on?
3. Any errors or issues?
4. Screenshots (if possible)?

I'll guide you through each step! 🚀
