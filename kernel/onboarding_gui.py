import eel
import sys
import os
from pathlib import Path
from typing import Dict, Any, Optional
import bottle # Import bottle

# --- Configuration and Video Management (can be reused from previous version) ---
class OnboardingConfig:
    """Manages onboarding configuration and state"""
    
    CONFIG_FILE = Path.home() / ".aios" / "onboarding_config.json"
    
    def __init__(self):
        self.config = self.load_config()
    
    def load_config(self) -> Dict[str, Any]:
        """Load existing configuration or create default"""
        # Simplified for Eel example, will expand later
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
        # Simplified for Eel example
        print("Saving config (Eel placeholder)")
    
    def mark_step_complete(self, step: int):
        """Mark a step as completed"""
        if step not in self.config["completed_steps"]:
            self.config["completed_steps"].append(step)
        self.save_config()
    
    def is_step_complete(self, step: int) -> bool:
        """Check if step is completed"""
        return step in self.config.get("completed_steps", [])


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
                # Return a URL that the Bottle route will handle
                return f"/assets/onboarding_videos/{cls.VIDEOS[video_key]['file']}"
        return None
    
    @classmethod
    def get_video_info(cls, video_key: str) -> Optional[Dict[str, str]]:
        """Get information about a video"""
        return cls.VIDEOS.get(video_key)

# --- Eel Application Setup ---
_AIOS_ROOT = Path(__file__).resolve().parent.parent
WEB_FOLDER = str(_AIOS_ROOT / 'web')
eel.init(WEB_FOLDER)

# This is a more direct way to serve static files with Bottle
# It needs to be done before eel.start()
@bottle.route('/assets/<filename:path>')
def serve_static_assets(filename):
    return bottle.static_file(filename, root=str(_AIOS_ROOT / "assets"))

# Global instance of config
onboarding_config = OnboardingConfig()

# Define the steps for the onboarding process
ONBOARDING_STEPS = [
    {"key": "welcome", "title": "Welcome to AI-OS", "description": "Your AI-Powered Operating System"},
    {"key": "hardware_setup", "title": "Hardware Detection", "description": "Scanning your system hardware"},
    {"key": "gpu_config", "title": "GPU Configuration", "description": "Setting up GPU acceleration"},
    {"key": "system", "title": "System Configuration", "description": "Basic system settings"},
    {"key": "complete", "title": "Setup Complete!", "description": "You're ready to launch AI-OS"}
]

@eel.expose
def get_current_step_data():
    """Returns data for the current onboarding step."""
    current_step_index = onboarding_config.config["current_step"]
    if 0 <= current_step_index < len(ONBOARDING_STEPS):
        step_data = ONBOARDING_STEPS[current_step_index]
        video_path = AIAssistantVideos.get_video_path(step_data["key"])
        return {
            "index": current_step_index,
            "total_steps": len(ONBOARDING_STEPS),
            "title": step_data["title"],
            "description": step_data["description"],
            "video_url": video_path
        }
    return None

@eel.expose
def next_step():
    """Advances to the next onboarding step."""
    current_step_index = onboarding_config.config["current_step"]
    if current_step_index < len(ONBOARDING_STEPS) - 1:
        onboarding_config.config["current_step"] += 1
        onboarding_config.mark_step_complete(current_step_index)
        onboarding_config.save_config()
        return get_current_step_data()
    return None

@eel.expose
def previous_step():
    """Goes back to the previous onboarding step."""
    current_step_index = onboarding_config.config["current_step"]
    if current_step_index > 0:
        onboarding_config.config["current_step"] -= 1
        onboarding_config.save_config()
        return get_current_step_data()
    return None

def start_eel_app():
    try:
        eel.start('index.html', size=(900, 700), port=8000)
    except Exception as e:
        print(f"Eel failed to start: {e}")
        print("Attempting to open in a regular web browser...")
        eel.start('index.html', size=(900, 700), mode=None, port=8000)

if __name__ == '__main__':
    start_eel_app()