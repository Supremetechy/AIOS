#!/usr/bin/env python3
"""
AI-OS Onboarding Launcher
Starts the Eel-based onboarding wizard
"""

import sys
import os
from pathlib import Path

# Ensure the project root is importable
sys.path.insert(0, str(Path(__file__).parent))


def check_dependencies():
    """Check if required dependencies are installed"""
    missing = []

    try:
        import eel
    except ImportError:
        missing.append("eel")

    try:
        import bottle
    except ImportError:
        missing.append("bottle")

    return missing


def main():
    """Main entry point"""
    print("=" * 60)
    print("AI-OS Onboarding Wizard")
    print("=" * 60)
    print()

    # Check dependencies
    missing = check_dependencies()

    if missing:
        print(f"Missing dependencies: {', '.join(missing)}")
        print(f"\nInstall with:")
        print(f"   pip install {' '.join(missing)}")
        sys.exit(1)

    # Verify web folder exists
    web_folder = Path(__file__).parent / "web"
    if not web_folder.is_dir():
        print(f"Web folder not found at: {web_folder}")
        print("The onboarding UI requires the web/ directory.")
        sys.exit(1)

    print("Starting onboarding wizard...")
    print()

    from kernel.onboarding_gui import start_eel_app
    start_eel_app()


if __name__ == "__main__":
    main()
