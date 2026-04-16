#!/usr/bin/env python3
"""
AI-OS Onboarding GUI with AI Talking Head Video Support
Main onboarding interface for new users
"""

import sys
import json
import os
import threading
from pathlib import Path
from typing import Dict, List, Optional, Any

try:
    from PyQt6.QtWidgets import (
        QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
        QLabel, QPushButton, QStackedWidget, QProgressBar, QCheckBox,
        QLineEdit, QTextEdit, QComboBox, QGroupBox, QScrollArea,
        QMessageBox, QFileDialog, QFrame
    )
    from PyQt6.QtCore import Qt, QTimer, QThread, pyqtSignal, QUrl, QSize
    from PyQt6.QtGui import QFont, QPixmap, QIcon, QPalette, QColor
    from PyQt6.QtMultimedia import QMediaPlayer, QAudioOutput
    from PyQt6.QtMultimediaWidgets import QVideoWidget
except ImportError:
    print("PyQt6 not installed. Install with: pip install PyQt6 PyQt6-Multimedia")
    sys.exit(1)

try:
    from kernel.voice_assistant import VoiceOnboardingAssistant, parse_command
    VOICE_AVAILABLE = True
except Exception as _voice_err:
    VoiceOnboardingAssistant = None  # type: ignore[assignment]
    parse_command = None  # type: ignore[assignment]
    VOICE_AVAILABLE = False
    print(f"[VOICE] Voice assistant unavailable: {_voice_err}")


class OnboardingConfig:
    """Manages onboarding configuration and state"""
    
    CONFIG_FILE = Path.home() / ".aios" / "onboarding_config.json"
    
    def __init__(self):
        self.config = self.load_config()
    
    def load_config(self) -> Dict[str, Any]:
        """Load existing configuration or create default"""
        if self.CONFIG_FILE.exists():
            with open(self.CONFIG_FILE, 'r') as f:
                return json.load(f)
        return {
            "onboarding_complete": False,
            "current_step": 0,
            "user_name": "",
            "system_name": "",
            "install_path": str(Path.cwd()),
            "gpu_enabled": False,
            "auto_start": False,
            "telemetry_enabled": True,
            "completed_steps": []
        }
    
    def save_config(self):
        """Save configuration to disk"""
        self.CONFIG_FILE.parent.mkdir(parents=True, exist_ok=True)
        with open(self.CONFIG_FILE, 'w') as f:
            json.dump(self.config, f, indent=2)
    
    def mark_step_complete(self, step: int):
        """Mark a step as completed"""
        if step not in self.config["completed_steps"]:
            self.config["completed_steps"].append(step)
        self.save_config()
    
    def is_step_complete(self, step: int) -> bool:
        """Check if step is completed"""
        return step in self.config.get("completed_steps", [])


class VideoPlayer(QWidget):
    """AI Talking Head Video Player Component"""

    def __init__(self, placeholder_text: str = "AI Assistant Video", parent=None):
        super().__init__(parent)
        self.media_player = QMediaPlayer()
        self.audio_output = QAudioOutput()
        self.media_player.setAudioOutput(self.audio_output)
        self.placeholder_text = placeholder_text
        self._has_video = False

        self.setup_ui()

    def setup_ui(self):
        """Setup video player UI"""
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)

        # Placeholder shown when no video is available
        self.placeholder = QFrame()
        self.placeholder.setMinimumHeight(200)
        self.placeholder.setStyleSheet("""
            QFrame {
                background-color: #1a2a3a;
                border-radius: 8px;
                border: 2px solid #2c3e50;
            }
        """)
        placeholder_layout = QVBoxLayout(self.placeholder)
        placeholder_layout.setAlignment(Qt.AlignmentFlag.AlignCenter)

        icon_label = QLabel("🤖")
        icon_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        icon_label.setStyleSheet("font-size: 48pt; border: none; background: transparent;")
        placeholder_layout.addWidget(icon_label)

        text_label = QLabel(self.placeholder_text)
        text_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        text_label.setStyleSheet("color: #7f8c8d; font-size: 11pt; border: none; background: transparent;")
        placeholder_layout.addWidget(text_label)

        layout.addWidget(self.placeholder)

        # Video widget (hidden until a video is loaded)
        self.video_widget = QVideoWidget()
        self.video_widget.setMinimumSize(QSize(640, 360))
        self.media_player.setVideoOutput(self.video_widget)
        self.video_widget.setVisible(False)
        layout.addWidget(self.video_widget)

        # Controls (hidden until a video is loaded)
        self.controls_widget = QWidget()
        controls_layout = QHBoxLayout(self.controls_widget)

        self.play_button = QPushButton("▶ Play")
        self.play_button.clicked.connect(self.toggle_play)

        self.stop_button = QPushButton("⏹ Stop")
        self.stop_button.clicked.connect(self.stop)

        self.progress_bar = QProgressBar()
        self.progress_bar.setRange(0, 100)

        controls_layout.addWidget(self.play_button)
        controls_layout.addWidget(self.stop_button)
        controls_layout.addWidget(self.progress_bar)

        self.controls_widget.setVisible(False)
        layout.addWidget(self.controls_widget)

        # Connect signals
        self.media_player.positionChanged.connect(self.update_progress)
        self.media_player.durationChanged.connect(self.update_duration)
        self.media_player.playbackStateChanged.connect(self.handle_state_change)

    def load_video(self, video_path: str):
        """Load a video file"""
        if os.path.exists(video_path):
            self.media_player.setSource(QUrl.fromLocalFile(video_path))
            self.placeholder.setVisible(False)
            self.video_widget.setVisible(True)
            self.controls_widget.setVisible(True)
            self._has_video = True
            self.play()
        else:
            print(f"Video not found: {video_path}")
            
    def toggle_play(self):
        """Toggle play/pause"""
        if self.media_player.playbackState() == QMediaPlayer.PlaybackState.PlayingState:
            self.media_player.pause()
        else:
            self.media_player.play()
    
    def play(self):
        """Start playing"""
        self.media_player.play()
        
    def stop(self):
        """Stop playing"""
        self.media_player.stop()
        
    def update_progress(self, position):
        """Update progress bar"""
        if self.media_player.duration() > 0:
            progress = int((position / self.media_player.duration()) * 100)
            self.progress_bar.setValue(progress)
    
    def update_duration(self, duration):
        """Update when duration changes"""
        self.progress_bar.setRange(0, 100)
        
    def handle_state_change(self, state):
        """Handle playback state changes"""
        if state == QMediaPlayer.PlaybackState.PlayingState:
            self.play_button.setText("⏸ Pause")
        else:
            self.play_button.setText("▶ Play")


class AIAssistantVideos:
    """Manages AI talking head video resources"""
    
    VIDEOS_DIR = Path("assets/onboarding_videos")
    
    VIDEOS = {
        "welcome": {
            "file": "welcome.mp4",
            "title": "Welcome to AI-OS",
            "description": "Introduction to your new AI-powered operating system"
        },
        "hardware_setup": {
            "file": "hardware_setup.mp4",
            "title": "Hardware Detection & Setup",
            "description": "Learn how AI-OS detects and optimizes your hardware"
        },
        "gpu_config": {
            "file": "gpu_configuration.mp4",
            "title": "GPU Configuration",
            "description": "Setting up GPU acceleration for AI workloads"
        },
        "network_setup": {
            "file": "network_setup.mp4",
            "title": "Network Configuration",
            "description": "Configure networking for distributed AI training"
        },
        "security": {
            "file": "security_overview.mp4",
            "title": "Security & Privacy",
            "description": "Understanding AI-OS security features"
        },
        "complete": {
            "file": "setup_complete.mp4",
            "title": "Setup Complete!",
            "description": "You're ready to start using AI-OS"
        }
    }
    
    @classmethod
    def get_video_path(cls, video_key: str) -> Optional[str]:
        """Get path to a video file"""
        if video_key in cls.VIDEOS:
            path = cls.VIDEOS_DIR / cls.VIDEOS[video_key]["file"]
            if path.exists():
                return str(path)
        return None
    
    @classmethod
    def get_video_info(cls, video_key: str) -> Optional[Dict[str, str]]:
        """Get information about a video"""
        return cls.VIDEOS.get(video_key)


class WelcomeStep(QWidget):
    """Welcome screen with AI talking head introduction"""
    
    def __init__(self, config: OnboardingConfig, parent=None):
        super().__init__(parent)
        self.config = config
        self.setup_ui()
        
    def setup_ui(self):
        """Setup welcome step UI"""
        layout = QVBoxLayout(self)
        
        # Title
        title = QLabel("Welcome to AI-OS")
        title.setFont(QFont("Arial", 24, QFont.Weight.Bold))
        title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(title)
        
        # Subtitle
        subtitle = QLabel("Your AI-Powered Operating System")
        subtitle.setFont(QFont("Arial", 14))
        subtitle.setAlignment(Qt.AlignmentFlag.AlignCenter)
        subtitle.setStyleSheet("color: #666; margin-bottom: 20px;")
        layout.addWidget(subtitle)
        
        # Welcome message
        message = QLabel(
            "Watch this short introduction to learn about AI-OS features and "
            "how to get started with your setup."
        )
        message.setWordWrap(True)
        message.setAlignment(Qt.AlignmentFlag.AlignCenter)
        message.setStyleSheet("margin: 20px; font-size: 12pt;")
        layout.addWidget(message)


class HardwareDetectionStep(QWidget):
    """Hardware detection and configuration step"""
    
    detection_complete = pyqtSignal(dict)
    
    def __init__(self, config: OnboardingConfig, parent=None):
        super().__init__(parent)
        self.config = config
        self.hardware_info = {}
        self.setup_ui()
        
    def setup_ui(self):
        """Setup hardware detection UI"""
        layout = QVBoxLayout(self)
        
        # Title
        title = QLabel("Hardware Detection")
        title.setFont(QFont("Arial", 20, QFont.Weight.Bold))
        layout.addWidget(title)
        
        # Detection status
        self.status_label = QLabel("Click 'Detect Hardware' to scan your system")
        self.status_label.setStyleSheet("font-size: 11pt; margin: 10px;")
        layout.addWidget(self.status_label)
        
        # Progress bar
        self.progress = QProgressBar()
        self.progress.setVisible(False)
        layout.addWidget(self.progress)
        
        # Hardware info display
        self.info_text = QTextEdit()
        self.info_text.setReadOnly(True)
        self.info_text.setMaximumHeight(200)
        layout.addWidget(self.info_text)
        
        # Detect button
        self.detect_button = QPushButton("🔍 Detect Hardware")
        self.detect_button.setMinimumHeight(40)
        self.detect_button.clicked.connect(self.start_detection)
        layout.addWidget(self.detect_button)
            
    def start_detection(self):
        """Start hardware detection"""
        self.progress.setVisible(True)
        self.progress.setRange(0, 0)  # Indeterminate
        self.detect_button.setEnabled(False)
        self.status_label.setText("Detecting hardware...")
        
        # Run detection in background
        QTimer.singleShot(100, self.run_detection)
        
    def run_detection(self):
        """Run actual hardware detection"""
        try:
            # Import hardware detection module
            from kernel.hardware_detection import HardwareDetector
            
            detector = HardwareDetector()
            specs = detector.detect_all()
            
            # Convert SystemSpecs to dictionary format for GUI
            cpus = [p for p in specs.processors if p.processor_type.value == 'cpu']
            gpus = [p for p in specs.processors if p.processor_type.value == 'gpu']
            
            self.hardware_info = {
                "cpu": cpus[0] if cpus else None,
                "gpus": gpus,
                "memory": specs.memory,
                "storage": specs.storage_devices,
                "specs": specs  # Keep full specs for later use
            }
            
            # Display results
            self.display_hardware_info()
            self.status_label.setText("✓ Hardware detection complete!")
            self.detection_complete.emit(self.hardware_info)
            
        except Exception as e:
            import traceback
            error_details = traceback.format_exc()
            self.status_label.setText(f"❌ Detection failed: {str(e)}")
            self.info_text.setPlainText(f"Error: {str(e)}\n\n{error_details}")
        
        finally:
            self.progress.setVisible(False)
            self.detect_button.setEnabled(True)
            
    def display_hardware_info(self):
        """Display detected hardware information"""
        info_lines = ["=== Detected Hardware ===\n"]
        
        if "cpu" in self.hardware_info and self.hardware_info["cpu"]:
            cpu = self.hardware_info["cpu"]
            info_lines.append(f"CPU: {cpu.model}")
            info_lines.append(f"Cores: {cpu.cores} | Threads: {cpu.threads}")
            info_lines.append(f"Vendor: {cpu.vendor.value.upper()}")
            info_lines.append("")
        
        if "gpus" in self.hardware_info and self.hardware_info["gpus"]:
            gpus = self.hardware_info["gpus"]
            info_lines.append("GPUs:")
            for i, gpu in enumerate(gpus, 1):
                info_lines.append(f"  {i}. {gpu.model}")
                info_lines.append(f"     Vendor: {gpu.vendor.value.upper()}")
                if gpu.memory_gb:
                    info_lines.append(f"     VRAM: {gpu.memory_gb:.2f} GB")
                if gpu.capabilities:
                    info_lines.append(f"     Capabilities: {', '.join(gpu.capabilities)}")
            info_lines.append("")
        
        if "memory" in self.hardware_info and self.hardware_info["memory"]:
            mem = self.hardware_info["memory"]
            info_lines.append(f"Memory:")
            info_lines.append(f"  Total: {mem.total_gb:.2f} GB")
            info_lines.append(f"  Used: {mem.used_gb:.2f} GB")
            info_lines.append(f"  Available: {mem.available_gb:.2f} GB")
            info_lines.append("")
            
        if "storage" in self.hardware_info and self.hardware_info["storage"]:
            storage = self.hardware_info["storage"]
            info_lines.append("Storage:")
            for disk in storage:
                storage_type = "SSD" if disk.is_ssd else "HDD"
                info_lines.append(f"  • {disk.device_name} ({storage_type})")
                info_lines.append(f"    {disk.total_gb:.2f} GB total, {disk.used_gb:.2f} GB used")
                info_lines.append(f"    Mount: {disk.mount_point}")
        
        self.info_text.setPlainText("\n".join(info_lines))


class GPUConfigurationStep(QWidget):
    """GPU configuration step"""
    
    def __init__(self, config: OnboardingConfig, hardware_info: Dict, parent=None):
        super().__init__(parent)
        self.config = config
        self.hardware_info = hardware_info
        self.setup_ui()
        
    def setup_ui(self):
        """Setup GPU configuration UI"""
        layout = QVBoxLayout(self)
        
        # Title
        title = QLabel("GPU Configuration")
        title.setFont(QFont("Arial", 20, QFont.Weight.Bold))
        layout.addWidget(title)
        
        # GPU options
        options_group = QGroupBox("GPU Settings")
        options_layout = QVBoxLayout()
            
    def auto_detect_gpus(self):
        """Auto-detect and enable appropriate GPU options"""
        gpus = self.hardware_info.get("gpu", [])
        
        for gpu in gpus:
            name = gpu.get("name", "").lower()
            if "nvidia" in name or "cuda" in name:
                self.cuda_checkbox.setChecked(True)
            elif "amd" in name or "radeon" in name:
                self.rocm_checkbox.setChecked(True)
            elif "apple" in name or "metal" in name:
                self.metal_checkbox.setChecked(True)


class SystemConfigurationStep(QWidget):
    """System configuration step"""
    
    def __init__(self, config: OnboardingConfig, parent=None):
        super().__init__(parent)
        self.config = config
        self.setup_ui()
        
    def setup_ui(self):
        """Setup system configuration UI"""
        layout = QVBoxLayout(self)
        
        # Title
        title = QLabel("System Configuration")
        title.setFont(QFont("Arial", 20, QFont.Weight.Bold))
        layout.addWidget(title)
        
        # Form
        form_group = QGroupBox("Basic Settings")
        form_layout = QVBoxLayout()
        
        # User name
        form_layout.addWidget(QLabel("Your Name:"))
        self.name_input = QLineEdit()
        self.name_input.setPlaceholderText("Enter your name")
        self.name_input.setText(self.config.config.get("user_name", ""))
        form_layout.addWidget(self.name_input)
        
        # System name
        form_layout.addWidget(QLabel("System Name:"))
        self.system_name_input = QLineEdit()
        self.system_name_input.setPlaceholderText("e.g., my-ai-workstation")
        self.system_name_input.setText(self.config.config.get("system_name", ""))
        form_layout.addWidget(self.system_name_input)
        
        # Install path
        form_layout.addWidget(QLabel("Installation Path:"))
        path_layout = QHBoxLayout()
        self.path_input = QLineEdit()
        self.path_input.setText(self.config.config.get("install_path", str(Path.cwd())))
        path_layout.addWidget(self.path_input)
        
        browse_button = QPushButton("Browse...")
        browse_button.clicked.connect(self.browse_path)
        path_layout.addWidget(browse_button)
        form_layout.addLayout(path_layout)
        
        # Auto-start option
        self.auto_start = QCheckBox("Start AI-OS automatically on boot")
        self.auto_start.setChecked(self.config.config.get("auto_start", False))
        form_layout.addWidget(self.auto_start)
        
        # Telemetry option
        self.telemetry = QCheckBox("Send anonymous usage statistics (helps improve AI-OS)")
        self.telemetry.setChecked(self.config.config.get("telemetry_enabled", True))
        form_layout.addWidget(self.telemetry)
        
        form_group.setLayout(form_layout)
        layout.addWidget(form_group)
        
    def browse_path(self):
        """Browse for installation path"""
        path = QFileDialog.getExistingDirectory(self, "Select Installation Directory")
        if path:
            self.path_input.setText(path)
            
    def get_configuration(self) -> Dict[str, Any]:
        """Get current configuration"""
        return {
            "user_name": self.name_input.text(),
            "system_name": self.system_name_input.text(),
            "install_path": self.path_input.text(),
            "auto_start": self.auto_start.isChecked(),
            "telemetry_enabled": self.telemetry.isChecked()
        }


class CompletionStep(QWidget):
    """Setup completion step"""
    
    def __init__(self, config: OnboardingConfig, parent=None):
        super().__init__(parent)
        self.config = config
        self.setup_ui()
        
    def setup_ui(self):
        """Setup completion UI"""
        layout = QVBoxLayout(self)
        
        # Title
        title = QLabel("🎉 Setup Complete!")
        title.setFont(QFont("Arial", 24, QFont.Weight.Bold))
        title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(title)
        
        # Completion message
        message = QLabel(
            "Congratulations! AI-OS is now configured and ready to use.\n\n"
            "You can now start running AI workloads, training models, and "
            "leveraging the full power of your hardware."
        )
        message.setWordWrap(True)
        message.setAlignment(Qt.AlignmentFlag.AlignCenter)
        message.setStyleSheet("font-size: 12pt; margin: 20px;")
        layout.addWidget(message)


class VoiceGuideController:
    """Bridges VoiceOnboardingAssistant to the PyQt onboarding wizard.

    Each time the wizard changes step, the controller speaks the step's
    prompt and listens for a voice command. Recognized commands ('next',
    'back', 'repeat', 'skip', 'help') are routed back to the wizard via
    Qt signals so UI updates stay on the main thread.
    """

    STEP_KEYS = ["welcome", "hardware", "gpu", "network", "complete"]

    def __init__(self, wizard, enabled: bool = True):
        self.wizard = wizard
        self.enabled = enabled and VOICE_AVAILABLE
        self.assistant = None
        self._listen_thread = None
        self._stop = False
        if self.enabled:
            try:
                self.assistant = VoiceOnboardingAssistant()
            except Exception as err:
                print(f"[VOICE] Failed to initialize assistant: {err}")
                self.enabled = False

    def announce_step(self, key: str, title: str, description: str) -> None:
        if not self.enabled or not self.assistant:
            return
        def _speak():
            try:
                self.assistant.speak(f"{title}. {description}")
            except Exception as err:
                print(f"[VOICE] speak error: {err}")
        t = threading.Thread(target=_speak, daemon=True, name="aios-voice-speak")
        t.start()

    def start_listening(self) -> None:
        if not self.enabled or not self.assistant:
            return
        if self._listen_thread and self._listen_thread.is_alive():
            return
        self._stop = False
        self._listen_thread = threading.Thread(
            target=self._listen_loop, daemon=True, name="aios-voice-listen",
        )
        self._listen_thread.start()

    def stop(self) -> None:
        self._stop = True
        if self.assistant:
            try:
                self.assistant.stop()
            except Exception:
                pass

    def _listen_loop(self) -> None:
        if not self.assistant:
            return
        while not self._stop:
            heard = self.assistant.listen(timeout=5.0)
            if heard is None:
                continue
            cmd = parse_command(heard) if parse_command else None
            if cmd:
                QTimer.singleShot(0, lambda c=cmd: self.wizard.handle_voice_command(c))


class OnboardingWizard(QMainWindow):
    """Main onboarding wizard window"""

    def __init__(self):
        super().__init__()
        self.config = OnboardingConfig()
        self.hardware_info = {}
        self.current_step = 0
        self.voice_guide = VoiceGuideController(self, enabled=VOICE_AVAILABLE)
        self.agent_video_player = VideoPlayer("AI Assistant", self) # Add this line

        self.setup_ui()
        self.setup_steps()
        # Announce first step and begin listening
        QTimer.singleShot(500, self._announce_current_step)
        self.voice_guide.start_listening()
        
    def setup_ui(self):
        """Setup main window UI"""
        self.setWindowTitle("AI-OS Setup Wizard")
        self.setMinimumSize(900, 700)
        
        # Center window on screen
        screen = QApplication.primaryScreen().geometry()
        x = (screen.width() - self.width()) // 2
        y = (screen.height() - self.height()) // 2
        self.move(x, y)
        
        # Main widget
        main_widget = QWidget()
        self.setCentralWidget(main_widget)
        
        # Main layout
        layout = QVBoxLayout(main_widget)
        
        # Header
        header = self.create_header()
        layout.addWidget(header)
        
        # Add the persistent video player here
        layout.addWidget(self.agent_video_player) # Add this line
        
        # Stacked widget for steps
        self.stack = QStackedWidget()
        layout.addWidget(self.stack, stretch=1)
        
        # Navigation buttons
        nav_layout = QHBoxLayout()
        
        self.back_button = QPushButton("← Back")
        self.back_button.clicked.connect(self.previous_step)
        self.back_button.setMinimumHeight(40)
        self.back_button.setEnabled(False)
        
        nav_layout.addWidget(self.back_button)
        nav_layout.addStretch()
        
        self.next_button = QPushButton("Next →")
        self.next_button.clicked.connect(self.next_step)
        self.next_button.setMinimumHeight(40)
        self.next_button.setMinimumWidth(120)
        
        self.finish_button = QPushButton("🚀 Launch AI-OS")
        self.finish_button.clicked.connect(self.finish_setup)
        self.finish_button.setMinimumHeight(40)
        self.finish_button.setMinimumWidth(120)
        self.finish_button.setVisible(False)
        
        nav_layout.addWidget(self.next_button)
        nav_layout.addWidget(self.finish_button)
        
        layout.addLayout(nav_layout)
        
        # Apply styling
        self.apply_styling()
        
    def create_header(self) -> QWidget:
        """Create header with progress indicator"""
        header = QWidget()
        header.setFixedHeight(80)
        header.setStyleSheet("background-color: #2c3e50; border-radius: 5px;")
        
        layout = QVBoxLayout(header)
        
        # Progress bar
        self.progress_bar = QProgressBar()
        self.progress_bar.setRange(0, 100)
        self.progress_bar.setValue(0)
        self.progress_bar.setTextVisible(True)
        self.progress_bar.setStyleSheet("""
            QProgressBar {
                border: 2px solid #3498db;
                border-radius: 5px;
                text-align: center;
                background-color: white;
                height: 25px;
            }
            QProgressBar::chunk {
                background-color: #3498db;
            }
        """)
        
        # Step indicator
        self.step_label = QLabel("Step 1 of 5: Welcome")
        self.step_label.setStyleSheet("color: white; font-size: 14pt; font-weight: bold;")
        self.step_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        
        layout.addWidget(self.step_label)
        layout.addWidget(self.progress_bar)
        
        return header
        
    def setup_steps(self):
        """Setup wizard steps"""
        # Step 1: Welcome
        self.welcome_step = WelcomeStep(self.config)
        self.stack.addWidget(self.welcome_step)
        
        # Step 2: Hardware Detection
        self.hardware_step = HardwareDetectionStep(self.config)
        self.hardware_step.detection_complete.connect(self.on_hardware_detected)
        self.stack.addWidget(self.hardware_step)
        
        # Step 3: GPU Configuration (placeholder, will be created after hardware detection)
        self.gpu_step = None
        
        # Step 4: System Configuration
        self.system_step = SystemConfigurationStep(self.config)
        self.stack.addWidget(self.system_step)
        
        # Step 5: Completion
        self.completion_step = CompletionStep(self.config)
        self.stack.addWidget(self.completion_step)
        
        self.update_navigation()
        
    def on_hardware_detected(self, hardware_info: Dict):
        """Handle hardware detection completion"""
        self.hardware_info = hardware_info
        
        # Create GPU configuration step if not exists
        if self.gpu_step is None and self.stack.count() < 5:
            self.gpu_step = GPUConfigurationStep(self.config, hardware_info)
            self.stack.insertWidget(2, self.gpu_step)
            
    def next_step(self):
        """Move to next step"""
        # Validate current step
        if not self.validate_current_step():
            return

        # Save current step configuration
        self.save_current_step()

        # Move to next step
        if self.current_step < self.stack.count() - 1:
            self.current_step += 1
            self.stack.setCurrentIndex(self.current_step)
            self.update_navigation()
            self.config.mark_step_complete(self.current_step - 1)
            self._announce_current_step()

    def previous_step(self):
        """Move to previous step"""
        if self.current_step > 0:
            self.current_step -= 1
            self.stack.setCurrentIndex(self.current_step)
            self.update_navigation()
            self._announce_current_step()

    def handle_voice_command(self, command: str) -> None:
        """Route a recognized voice command to the wizard."""
        if command == "next":
            if self.current_step == self.stack.count() - 1:
                self.finish_setup()
            else:
                self.next_step()
        elif command == "back":
            self.previous_step()
        elif command == "repeat":
            self._announce_current_step()
        elif command == "skip":
            if self.current_step < self.stack.count() - 1:
                self.current_step += 1
                self.stack.setCurrentIndex(self.current_step)
                self.update_navigation()
                self._announce_current_step()
        elif command == "help":
            self._speak(
                "Available commands are: next, back, repeat, skip, and help. "
                "You can also just follow the on-screen prompts."
            )
        elif command == "exit":
            self.close()
        elif command == "detect" and isinstance(
            self.stack.currentWidget(), HardwareDetectionStep
        ):
            self.stack.currentWidget().start_detection()
        elif command == "launch" and self.current_step == self.stack.count() - 1:
            self.finish_setup()

    def _announce_current_step(self) -> None:
        """Speak the current step's title and prompt via the voice guide."""
        if not getattr(self, "voice_guide", None) or not self.voice_guide.enabled:
            return
        step_prompts = [
            ("welcome", "Welcome",
             "Watch the introduction. Say 'next' when you are ready to continue."),
            ("hardware", "Hardware Detection",
             "Say 'detect' to scan your hardware, or 'next' to skip."),
            ("gpu", "GPU Configuration",
             "Choose which GPU acceleration backends to enable, then say 'next'."),
            ("system", "System Configuration",
             "Enter your name and system name on screen, then say 'next'."),
            ("complete", "Setup Complete",
             "Say 'launch' to start AI-OS, or 'exit' to finish without launching."),
        ]
        if self.current_step < len(step_prompts):
            key, title, prompt = step_prompts[self.current_step]
            self.voice_guide.announce_step(key, title, prompt)

            # Load the corresponding video for the current step
            video_path = AIAssistantVideos.get_video_path(key)
            if video_path:
                self.agent_video_player.load_video(video_path)
            else:
                self.agent_video_player.stop() # Stop any playing video if no video for step
                print(f"No video found for step key: {key}") # For debugging

    def _speak(self, text: str) -> None:
        if getattr(self, "voice_guide", None) and self.voice_guide.enabled and self.voice_guide.assistant:
            t = threading.Thread(
                target=self.voice_guide.assistant.speak, args=(text,), daemon=True,
            )
            t.start()
            
    def validate_current_step(self) -> bool:
        """Validate current step before proceeding"""
        current_widget = self.stack.currentWidget()
        
        # Step 2: Ensure hardware detection is run
        if isinstance(current_widget, HardwareDetectionStep):
            if not current_widget.hardware_info:
                QMessageBox.warning(
                    self,
                    "Hardware Detection Required",
                    "Please run hardware detection before proceeding."
                )
                return False
                
        # Step 4: Ensure required fields are filled
        if isinstance(current_widget, SystemConfigurationStep):
            config = current_widget.get_configuration()
            if not config["user_name"] or not config["system_name"]:
                QMessageBox.warning(
                    self,
                    "Required Fields",
                    "Please fill in your name and system name."
                )
                return False
                
        return True
        
    def save_current_step(self):
        """Save current step configuration"""
        current_widget = self.stack.currentWidget()
        
        if isinstance(current_widget, SystemConfigurationStep):
            config = current_widget.get_configuration()
            self.config.config.update(config)
            self.config.save_config()
            
    def update_navigation(self):
        """Update navigation buttons and progress"""
        total_steps = self.stack.count()
        
        # Update progress bar
        progress = int((self.current_step / (total_steps - 1)) * 100)
        self.progress_bar.setValue(progress)
        
        # Update step label
        step_names = [
            "Welcome",
            "Hardware Detection",
            "GPU Configuration",
            "System Settings",
            "Complete"
        ]
        
        if self.current_step < len(step_names):
            step_name = step_names[self.current_step]
            self.step_label.setText(f"Step {self.current_step + 1} of {total_steps}: {step_name}")
        
        # Update buttons
        self.back_button.setEnabled(self.current_step > 0)
        
        if self.current_step == total_steps - 1:
            self.next_button.setVisible(False)
            self.finish_button.setVisible(True)
        else:
            self.next_button.setVisible(True)
            self.finish_button.setVisible(False)
            
    def closeEvent(self, event):  # type: ignore[override]
        if getattr(self, "voice_guide", None):
            self.voice_guide.stop()
        super().closeEvent(event)

    def finish_setup(self):
        """Finish setup and launch AI-OS"""
        # Mark onboarding as complete
        self.config.config["onboarding_complete"] = True
        self.config.save_config()
        if getattr(self, "voice_guide", None):
            self.voice_guide.stop()
        
        # Show success message
        msg = QMessageBox(self)
        msg.setIcon(QMessageBox.Icon.Information)
        msg.setWindowTitle("Setup Complete")
        msg.setText("AI-OS setup is complete!")
        msg.setInformativeText(
            "Would you like to launch AI-OS now or exit to configure manually?"
        )
        
        launch_button = msg.addButton("Launch Now", QMessageBox.ButtonRole.AcceptRole)
        exit_button = msg.addButton("Exit", QMessageBox.ButtonRole.RejectRole)
        
        msg.exec()
        
        if msg.clickedButton() == launch_button:
            self.launch_aios()
        
        self.close()
        
    def launch_aios(self):
        """Launch AI-OS kernel"""
        try:
            import subprocess
            subprocess.Popen([sys.executable, "aios_kernel.py"])
        except Exception as e:
            QMessageBox.critical(
                self,
                "Launch Failed",
                f"Failed to launch AI-OS: {str(e)}"
            )
            
    def apply_styling(self):
        """Apply application-wide styling"""
        self.setStyleSheet("""
            QMainWindow {
                background-color: #ecf0f1;
            }
            QPushButton {
                background-color: #3498db;
                color: white;
                border: none;
                border-radius: 5px;
                padding: 8px 16px;
                font-size: 11pt;
                font-weight: bold;
            }
            QPushButton:hover {
                background-color: #2980b9;
            }
            QPushButton:disabled {
                background-color: #95a5a6;
            }
            QGroupBox {
                font-weight: bold;
                border: 2px solid #bdc3c7;
                border-radius: 5px;
                margin-top: 10px;
                padding-top: 10px;
            }
            QGroupBox::title {
                subcontrol-origin: margin;
                left: 10px;
                padding: 0 5px;
            }
            QLineEdit, QTextEdit {
                border: 2px solid #bdc3c7;
                border-radius: 5px;
                padding: 5px;
                font-size: 10pt;
            }
            QLineEdit:focus, QTextEdit:focus {
                border: 2px solid #3498db;
            }
            QCheckBox {
                font-size: 10pt;
                spacing: 5px;
            }
        """)


def main():
    """Main entry point"""
    app = QApplication(sys.argv)
    
    # Set application info
    app.setApplicationName("AI-OS Setup")
    app.setOrganizationName("AI-OS")
    
    # Create and show wizard
    wizard = OnboardingWizard()
    wizard.show()
    
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
