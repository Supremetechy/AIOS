# AI-OS Onboarding System Guide

## Overview

The AI-OS Onboarding System provides a beautiful, user-friendly GUI wizard that guides new users through the initial setup process. Each step includes an **AI talking head video** that provides detailed instructions and insights.

## Features

### 🎥 AI Talking Head Videos
- Pre-recorded AI assistant videos at each step
- Professional guidance and explanations
- Automatic playback with user controls

### 🖥️ Step-by-Step Wizard
1. **Welcome** - Introduction to AI-OS
2. **Hardware Detection** - Automatic system scanning
3. **GPU Configuration** - GPU optimization setup
4. **System Configuration** - User preferences and settings
5. **Completion** - Setup summary and next steps

### ✨ User Experience
- Modern, intuitive interface
- Progress tracking
- Form validation
- Configuration persistence
- Direct integration with AI-OS kernel

## Installation

### 1. Install Dependencies

```bash
pip install PyQt6 PyQt6-Multimedia
```

### 2. Prepare Video Assets (Optional)

For testing, generate placeholder videos:

```bash
python tools/generate_demo_videos.py
```

Or add professional AI talking head videos to `assets/onboarding_videos/`:
- `welcome.mp4`
- `hardware_setup.mp4`
- `gpu_configuration.mp4`
- `network_setup.mp4`
- `security_overview.mp4`
- `setup_complete.mp4`

See `assets/onboarding_videos/README.md` for video specifications and creation guide.

## Usage

### Launch Onboarding Wizard

```bash
python run_onboarding.py
```

Or directly:

```bash
python -m kernel.onboarding_gui
```

### Configuration Storage

User configuration is saved to:
```
~/.aios/onboarding_config.json
```

## Onboarding Steps in Detail

### Step 1: Welcome
- Plays welcome video introducing AI-OS
- Overview of features and capabilities
- Sets user expectations

### Step 2: Hardware Detection
- Detects CPU, GPU, memory, and storage
- Integrates with `kernel.hardware_detection` module
- Displays detected hardware specifications
- Video explains hardware optimization

### Step 3: GPU Configuration
- Auto-detects GPU type (NVIDIA, AMD, Apple)
- Enables appropriate acceleration (CUDA, ROCm, Metal)
- User can customize GPU settings
- Video demonstrates GPU benefits

### Step 4: System Configuration
- User name and system name input
- Installation path selection
- Auto-start preferences
- Telemetry opt-in/out
- Video explains configuration options

### Step 5: Completion
- Summary of setup
- Option to launch AI-OS immediately
- Links to documentation
- Video congratulates and provides next steps

## Architecture

### Components

#### `OnboardingConfig`
- Manages configuration state
- Persists settings to JSON
- Tracks completed steps

#### `VideoPlayer`
- Qt6 multimedia video player
- Playback controls (play, pause, stop)
- Progress tracking
- Supports MP4 format

#### `AIAssistantVideos`
- Video resource manager
- Maps video keys to files
- Provides video metadata

#### Step Widgets
- `WelcomeStep` - Welcome screen
- `HardwareDetectionStep` - Hardware scanning
- `GPUConfigurationStep` - GPU setup
- `SystemConfigurationStep` - User preferences
- `CompletionStep` - Final step

#### `OnboardingWizard`
- Main window controller
- Navigation logic
- Step validation
- Configuration saving
- Launch integration

## Customization

### Adding New Steps

1. Create a new step widget:

```python
class CustomStep(QWidget):
    def __init__(self, config: OnboardingConfig, parent=None):
        super().__init__(parent)
        self.config = config
        self.setup_ui()
    
    def setup_ui(self):
        layout = QVBoxLayout(self)
        # Add your UI components
```

2. Add to wizard in `OnboardingWizard.setup_steps()`:

```python
self.custom_step = CustomStep(self.config)
self.stack.insertWidget(position, self.custom_step)
```

3. Update step names in `update_navigation()`.

### Customizing Styling

Modify `OnboardingWizard.apply_styling()` to change colors, fonts, and layout:

```python
self.setStyleSheet("""
    QPushButton {
        background-color: #your-color;
        /* ... */
    }
""")
```

### Adding Video Content

Replace placeholder videos with professional AI talking head videos:

1. Use services like D-ID, Synthesia, or HeyGen
2. Generate videos with scripts from `assets/onboarding_videos/README.md`
3. Place MP4 files in `assets/onboarding_videos/`
4. Ensure filenames match the expected names

## Integration with AI-OS Kernel

The onboarding system integrates with existing kernel modules:

### Hardware Detection
```python
from kernel.hardware_detection import HardwareDetection
detector = HardwareDetection()
hardware_info = detector.detect_cpu()
```

### Launch AI-OS
```python
import subprocess
subprocess.Popen([sys.executable, "aios_kernel.py"])
```

### Configuration Usage

Other modules can read onboarding configuration:

```python
from kernel.onboarding_gui import OnboardingConfig

config = OnboardingConfig()
user_name = config.config.get("user_name")
gpu_enabled = config.config.get("gpu_enabled")
```

## Testing

### Test Without Videos

The GUI works without videos - players show empty frames:

```bash
python run_onboarding.py
```

### Test with Placeholder Videos

Generate and test with simple placeholder videos:

```bash
python tools/generate_demo_videos.py
python run_onboarding.py
```

### Test Hardware Detection

Ensure hardware detection works:

```python
from kernel.hardware_detection import HardwareDetection
detector = HardwareDetection()
print(detector.detect_cpu())
print(detector.detect_gpu())
```

## Troubleshooting

### PyQt6 Import Error

```bash
pip install PyQt6 PyQt6-Multimedia
```

### Video Not Playing

1. Check video file exists in `assets/onboarding_videos/`
2. Verify MP4 format with H.264 codec
3. Check file permissions
4. Test video with system player

### Hardware Detection Fails

1. Ensure `kernel.hardware_detection` module is available
2. Check system permissions for hardware access
3. Review error messages in console

### Configuration Not Saving

1. Check write permissions for `~/.aios/` directory
2. Verify disk space
3. Check for JSON syntax errors

## Production Deployment

### Pre-launch Checklist

- [ ] Replace placeholder videos with professional AI talking head videos
- [ ] Test on target platforms (Windows, macOS, Linux)
- [ ] Verify hardware detection on various systems
- [ ] Update video scripts with actual product information
- [ ] Add error handling for edge cases
- [ ] Implement analytics/telemetry (if enabled)
- [ ] Create desktop shortcuts/launchers
- [ ] Add uninstaller option

### Performance Optimization

- Compress videos to reduce file size (target < 10MB per video)
- Lazy-load video players
- Cache hardware detection results
- Minimize startup time

### Accessibility

- Add keyboard navigation
- Support screen readers
- Provide text transcripts for videos
- Add high-contrast theme option

## Future Enhancements

- [ ] Multi-language support
- [ ] Interactive tutorials
- [ ] Live AI assistant (real-time TTS)
- [ ] Cloud sync for configuration
- [ ] Advanced troubleshooting wizard
- [ ] Performance benchmarking step
- [ ] Model marketplace integration
- [ ] Community connection features

## API Reference

See inline documentation in `kernel/onboarding_gui.py` for detailed API reference.

## Support

For issues or questions:
- Check `README_AIOS.md` for general documentation
- Review `GETTING_STARTED.md` for quick start guide
- See troubleshooting section above

## License

Same as AI-OS project license.
