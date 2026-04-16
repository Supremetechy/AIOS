# AI-OS Onboarding - Quick Start

## 🚀 Get Started in 3 Steps

### 1. Install Dependencies

```bash
pip install -r requirements_gui.txt
```

This installs:
- PyQt6 (GUI framework)
- PyQt6-Multimedia (video playback)

### 2. Run the Wizard

```bash
python run_onboarding.py
```

### 3. Follow the Steps

The wizard will guide you through:

1. **Welcome** - Watch intro video, learn about AI-OS
2. **Hardware Detection** - Click "Detect Hardware" to scan your system
3. **GPU Configuration** - Enable GPU acceleration (auto-detected)
4. **System Settings** - Enter your name and preferences
5. **Complete** - Launch AI-OS!

## 📹 About the AI Talking Head Videos

Each step includes an AI assistant video that:
- Explains what's happening
- Provides setup guidance
- Gives tips and best practices

### Using Placeholder Videos (For Testing)

If you don't have professional videos yet:

```bash
# Generate simple placeholder videos
python tools/generate_demo_videos.py
```

This requires `ffmpeg`:
- **macOS**: `brew install ffmpeg`
- **Linux**: `sudo apt-get install ffmpeg`

### Using Professional Videos

For production, add professional AI talking head videos to:
```
assets/onboarding_videos/
├── welcome.mp4
├── hardware_setup.mp4
├── gpu_configuration.mp4
├── network_setup.mp4
├── security_overview.mp4
└── setup_complete.mp4
```

See `assets/onboarding_videos/README.md` for video creation services and specifications.

## ⚙️ Configuration

Your settings are saved to:
```
~/.aios/onboarding_config.json
```

This includes:
- User name and system name
- Hardware configuration
- GPU preferences
- Auto-start settings

## 🎯 After Onboarding

Once complete, you can:
- Launch AI-OS directly from the wizard
- Run `python aios_kernel.py` manually
- Check `GETTING_STARTED.md` for tutorials
- Explore AI workloads and model management

## 🐛 Troubleshooting

### "PyQt6 not found"
```bash
pip install PyQt6 PyQt6-Multimedia
```

### Videos don't play
- Onboarding works without videos (video players just show placeholders)
- Generate test videos: `python tools/generate_demo_videos.py`
- Or add professional MP4 videos to `assets/onboarding_videos/`

### Hardware detection fails
- Ensure `kernel/hardware_detection.py` is accessible
- Check system permissions
- Try manual setup mode

## 📚 More Information

- **Full Guide**: `ONBOARDING_GUIDE.md`
- **AI-OS Docs**: `README_AIOS.md`
- **Getting Started**: `GETTING_STARTED.md`

## 🎬 Demo Mode

Want to test without setup?

```bash
# Skip onboarding and run demo
python demo.py
```

---

**Ready?** Run `python run_onboarding.py` to begin! 🚀
