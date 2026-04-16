#!/usr/bin/env python3
"""
AI-OS Onboarding Launcher
Starts the GUI-based onboarding wizard
"""

import sys
import os
from pathlib import Path

# Add kernel directory to path
sys.path.insert(0, str(Path(__file__).parent))

def check_dependencies():
    """Check if required dependencies are installed"""
    has_pyqt6 = False
    has_multimedia = False
    
    try:
        import PyQt6
        has_pyqt6 = True
    except ImportError:
        pass
    
    try:
        from PyQt6 import QtMultimedia
        has_multimedia = True
    except ImportError:
        pass
    
    return has_pyqt6, has_multimedia


def main():
    """Main entry point"""
    print("=" * 60)
    print("AI-OS Onboarding Wizard")
    print("=" * 60)
    print()
    
    # Check dependencies
    has_pyqt6, has_multimedia = check_dependencies()
    
    if not has_pyqt6:
        print("❌ PyQt6 not installed!")
        print("\nInstall with:")
        print("   pip install PyQt6")
        sys.exit(1)
    
    # Use fallback version if multimedia is not available
    if not has_multimedia:
        print("⚠️  PyQt6-Multimedia not available")
        print("   Using fallback version (video placeholders only)")
        print()
        from kernel.onboarding_gui_fallback import main as run_gui
    else:
        print("✓ Full video support available")
        print()
        from kernel.onboarding_gui import main as run_gui
    
    print("✓ Starting onboarding wizard...")
    print()
    
    run_gui()


if __name__ == "__main__":
    main()
