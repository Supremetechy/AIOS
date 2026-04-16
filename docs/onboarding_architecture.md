# AI-OS Onboarding System Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI-OS Onboarding Wizard                      │
│                     (PyQt6 GUI Application)                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ├──────────────────────────────────┐
                              │                                  │
                    ┌─────────▼─────────┐           ┌───────────▼──────────┐
                    │  Video Player     │           │  Configuration       │
                    │  Component        │           │  Manager             │
                    │  (QMediaPlayer)   │           │  (JSON Persistence)  │
                    └─────────┬─────────┘           └───────────┬──────────┘
                              │                                  │
                              │                                  │
                    ┌─────────▼─────────┐           ┌───────────▼──────────┐
                    │  AI Talking Head  │           │  ~/.aios/            │
                    │  Videos (MP4)     │           │  onboarding_config   │
                    │  6 video files    │           │  .json               │
                    └───────────────────┘           └──────────────────────┘
```

## Onboarding Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         User Starts Wizard                          │
│                      python run_onboarding.py                       │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 ▼
                    ┌────────────────────────┐
                    │   STEP 1: Welcome      │
                    │   ┌───────────────┐    │
                    │   │ 🎥 Welcome    │    │
                    │   │    Video      │    │
                    │   └───────────────┘    │
                    │   • Introduction       │
                    │   • Feature overview   │
                    └───────────┬────────────┘
                                │ [Next]
                                ▼
                    ┌────────────────────────┐
                    │ STEP 2: Hardware       │
                    │   ┌───────────────┐    │
                    │   │ 🎥 Hardware   │    │
                    │   │    Video      │    │
                    │   └───────────────┘    │
                    │   • Detect CPU         │
                    │   • Detect GPU         │
                    │   • Detect Memory      │
                    │   • Detect Storage     │
                    └───────────┬────────────┘
                                │ [Next]
                                ▼
                    ┌────────────────────────┐
                    │ STEP 3: GPU Config     │
                    │   ┌───────────────┐    │
                    │   │ 🎥 GPU Setup  │    │
                    │   │    Video      │    │
                    │   └───────────────┘    │
                    │   ☑ Enable CUDA        │
                    │   ☐ Enable ROCm        │
                    │   ☐ Enable Metal       │
                    └───────────┬────────────┘
                                │ [Next]
                                ▼
                    ┌────────────────────────┐
                    │ STEP 4: System Config  │
                    │   • User name          │
                    │   • System name        │
                    │   • Install path       │
                    │   • Auto-start         │
                    │   • Telemetry          │
                    └───────────┬────────────┘
                                │ [Next]
                                ▼
                    ┌────────────────────────┐
                    │ STEP 5: Complete       │
                    │   ┌───────────────┐    │
                    │   │ 🎥 Complete   │    │
                    │   │    Video      │    │
                    │   └───────────────┘    │
                    │   🎉 Setup Done!       │
                    │   • Next steps         │
                    │   • Documentation      │
                    └───────────┬────────────┘
                                │ [Launch AI-OS]
                                ▼
                    ┌────────────────────────┐
                    │   AI-OS Kernel Starts  │
                    │   aios_kernel.py       │
                    └────────────────────────┘
```

## Component Architecture

### 1. Video Player Component

```python
VideoPlayer (QWidget)
├── QVideoWidget (Display)
├── QMediaPlayer (Playback engine)
├── QAudioOutput (Audio)
└── Controls
    ├── Play/Pause button
    ├── Stop button
    └── Progress bar
```

### 2. Configuration System

```python
OnboardingConfig
├── load_config()
│   └── Read from ~/.aios/onboarding_config.json
├── save_config()
│   └── Write to ~/.aios/onboarding_config.json
├── mark_step_complete(step_id)
└── is_step_complete(step_id)
```

### 3. Step Widgets Hierarchy

```python
QStackedWidget
├── WelcomeStep
│   └── VideoPlayer (welcome.mp4)
├── HardwareDetectionStep
│   ├── VideoPlayer (hardware_setup.mp4)
│   ├── Status label
│   ├── Progress bar
│   ├── Info display
│   └── Detect button → HardwareDetection()
├── GPUConfigurationStep
│   ├── VideoPlayer (gpu_configuration.mp4)
│   └── GPU checkboxes (auto-detected)
├── SystemConfigurationStep
│   ├── Name input
│   ├── System name input
│   ├── Path selector
│   ├── Auto-start checkbox
│   └── Telemetry checkbox
└── CompletionStep
    ├── VideoPlayer (setup_complete.mp4)
    └── Next steps guide
```

## Data Flow

```
User Input → Step Widget → OnboardingConfig → JSON File
                ↓
          Validation
                ↓
          Navigation
                ↓
          Next Step → Video Player → AI Video
```

## File Structure

```
ai-os/
├── kernel/
│   ├── onboarding_gui.py          # Main GUI application (31 KB)
│   ├── hardware_detection.py      # Hardware detection integration
│   └── __init__.py
│
├── assets/
│   └── onboarding_videos/
│       ├── README.md              # Video creation guide
│       ├── welcome.mp4            # AI talking head videos
│       ├── hardware_setup.mp4
│       ├── gpu_configuration.mp4
│       ├── network_setup.mp4
│       ├── security_overview.mp4
│       └── setup_complete.mp4
│
├── tools/
│   └── generate_demo_videos.py    # FFmpeg video generator
│
├── docs/
│   └── onboarding_architecture.md # This file
│
├── run_onboarding.py              # Launcher script
├── test_onboarding.py             # Test suite
├── requirements_gui.txt           # PyQt6 dependencies
├── setup_onboarding.sh            # Automated setup
│
├── ONBOARDING_GUIDE.md            # Full documentation
├── QUICKSTART_ONBOARDING.md       # Quick start
└── ONBOARDING_SUMMARY.md          # Implementation summary
```

## Integration Points

### Hardware Detection Integration

```python
from kernel.hardware_detection import HardwareDetection

detector = HardwareDetection()
hardware_info = {
    "cpu": detector.detect_cpu(),
    "gpu": detector.detect_gpu(),
    "memory": detector.detect_memory(),
    "storage": detector.detect_storage()
}
```

### AI-OS Launch Integration

```python
import subprocess
import sys

subprocess.Popen([sys.executable, "aios_kernel.py"])
```

### Configuration Access

```python
from kernel.onboarding_gui import OnboardingConfig

config = OnboardingConfig()
if config.config.get("onboarding_complete"):
    # User has completed onboarding
    user_name = config.config.get("user_name")
```

## Technology Stack

```
┌─────────────────────────────────────┐
│         User Interface Layer        │
│           PyQt6 Widgets             │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│       Multimedia Layer              │
│   PyQt6 Multimedia (Video)          │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      Business Logic Layer           │
│   • Configuration Management        │
│   • Step Navigation                 │
│   • Validation                      │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      Integration Layer              │
│   • Hardware Detection              │
│   • AI-OS Kernel Launch             │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│       Persistence Layer             │
│   JSON Configuration Storage        │
└─────────────────────────────────────┘
```

## Video Resource Management

```python
AIAssistantVideos.VIDEOS = {
    "welcome": {
        "file": "welcome.mp4",
        "title": "Welcome to AI-OS",
        "description": "Introduction to AI-OS"
    },
    # ... 5 more videos
}

# Usage
video_path = AIAssistantVideos.get_video_path("welcome")
video_info = AIAssistantVideos.get_video_info("welcome")
```

## State Management

```
Configuration State (JSON):
{
    "onboarding_complete": false,
    "current_step": 0,
    "user_name": "",
    "system_name": "",
    "install_path": "/current/directory",
    "gpu_enabled": false,
    "auto_start": false,
    "telemetry_enabled": true,
    "completed_steps": []
}
```

## Navigation Logic

```python
def next_step():
    if validate_current_step():
        save_current_step()
        current_step += 1
        mark_step_complete(current_step - 1)
        update_ui()

def previous_step():
    if current_step > 0:
        current_step -= 1
        update_ui()
```

## Styling System

```css
/* QMainWindow styling */
background-color: #ecf0f1

/* QPushButton styling */
background-color: #3498db
color: white
border-radius: 5px

/* Progress bar */
border: 2px solid #3498db
background-color: #3498db (chunk)
```

## Error Handling

```python
try:
    detector = HardwareDetection()
    hardware_info = detector.detect_cpu()
except Exception as e:
    display_error(f"Detection failed: {e}")
    allow_skip_option()
```

## Cross-Platform Support

```
┌─────────────┬──────────────┬──────────────┐
│   macOS     │   Linux      │   Windows    │
├─────────────┼──────────────┼──────────────┤
│ Metal GPU   │ CUDA/ROCm    │ CUDA         │
│ .app bundle │ .desktop     │ .exe         │
│ DMG install │ .deb/.rpm    │ MSI install  │
└─────────────┴──────────────┴──────────────┘
```

## Performance Considerations

- **Lazy Loading**: Video players created only when step is shown
- **Async Detection**: Hardware detection runs in background
- **Caching**: Configuration cached in memory
- **Video Compression**: Target < 10MB per video file
- **Startup Time**: < 2 seconds from launch to first screen

## Security Considerations

- **Configuration Storage**: User home directory (~/.aios/)
- **File Permissions**: Config file readable only by user
- **Telemetry**: Opt-in only, clearly disclosed
- **Video Sources**: Validated MP4 files only
- **Input Validation**: All user inputs sanitized

## Accessibility Features

- Keyboard navigation (Tab/Enter)
- Clear visual hierarchy
- High contrast text
- Screen reader compatible labels
- Large touch targets (40px minimum)

## Future Architecture Extensions

```
┌────────────────────────────────────┐
│      Cloud Sync Layer (Future)    │
│   • Sync config across devices    │
│   • Cloud backup                  │
└────────────────────────────────────┘

┌────────────────────────────────────┐
│    Analytics Layer (Future)       │
│   • Usage tracking (opt-in)       │
│   • Error reporting               │
└────────────────────────────────────┘

┌────────────────────────────────────┐
│   AI Assistant Layer (Future)     │
│   • Live voice assistance         │
│   • Interactive Q&A               │
│   • Real-time TTS                 │
└────────────────────────────────────┘
```

## Testing Strategy

```
Unit Tests
├── Configuration save/load
├── Video resource management
├── Step validation logic
└── Navigation state machine

Integration Tests
├── Hardware detection integration
├── AI-OS launch integration
└── End-to-end wizard flow

UI Tests
├── Button click handlers
├── Form validation
├── Navigation flow
└── Video playback
```

## Deployment Pipeline

```
1. Development
   ├── Code implementation
   ├── Unit testing
   └── Local testing

2. Staging
   ├── Create professional videos
   ├── Integration testing
   └── User acceptance testing

3. Production
   ├── Package application
   ├── Create installers
   └── Deploy to users
```

This architecture provides a **scalable, maintainable, and user-friendly** onboarding experience with AI video guidance at every step.
