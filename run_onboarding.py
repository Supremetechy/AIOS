#!/usr/bin/env python3
"""
AI-OS Onboarding Launcher
Starts the unified Eel-based onboarding wizard with binary avatar support.
This is the single entry point — run_aios_with_avatar.py redirects here.
"""

import sys
import logging
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("AIOS")


def check_dependencies():
    missing = []
    for pkg in ("eel", "bottle", "websockets", "numpy", "psutil"):
        try:
            __import__(pkg)
        except ImportError:
            missing.append(pkg)
    return missing


def main():
    logger.info("=" * 60)
    logger.info("AIOS Onboarding — Binary Avatar Integration")
    logger.info("=" * 60)

    missing = check_dependencies()
    if missing:
        logger.error(f"Missing dependencies: {', '.join(missing)}")
        logger.info(f"Install with: pip install {' '.join(missing)}")
        sys.exit(1)

    web_folder = Path(__file__).parent / "web"
    if not web_folder.is_dir():
        logger.error(f"Web folder not found at: {web_folder}")
        sys.exit(1)

    logger.info("Starting unified onboarding wizard with avatar...")
    from kernel.onboarding_gui import start_eel_app
    start_eel_app()


if __name__ == "__main__":
    main()
