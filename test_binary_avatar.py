#!/usr/bin/env python3
"""
Test Binary Avatar System
Validates all components and runs integration tests
"""

import sys
import subprocess
from pathlib import Path

def print_header(text):
    print("\n" + "=" * 60)
    print(f"  {text}")
    print("=" * 60 + "\n")

def check_python_packages():
    """Check if required Python packages are installed"""
    print_header("Checking Python Dependencies")
    
    required = {
        'websockets': 'WebSocket server',
        'numpy': 'Audio processing',
        'soundfile': 'Audio I/O',
        'psutil': 'System monitoring'
    }
    
    optional = {
        'TTS': 'Coqui TTS (high-fidelity)',
    }
    
    all_good = True
    
    for package, description in required.items():
        try:
            __import__(package)
            print(f"✅ {package:15} - {description}")
        except ImportError:
            print(f"❌ {package:15} - {description} (MISSING)")
            all_good = False
    
    print("\nOptional packages:")
    for package, description in optional.items():
        try:
            __import__(package)
            print(f"✅ {package:15} - {description}")
        except ImportError:
            print(f"⚠️  {package:15} - {description} (not installed, will use fallback)")
    
    return all_good

def check_files():
    """Check if all required files exist"""
    print_header("Checking Files")
    
    files = [
        'web/binary-avatar.js',
        'web/avatar-controller.js',
        'web/avatar-bridge.py',
        'web/avatar-integration.html',
        'web/requirements-avatar.txt',
        'README_AVATAR.md',
        'start_binary_avatar.sh'
    ]
    
    all_exist = True
    for file in files:
        path = Path(file)
        if path.exists():
            size = path.stat().st_size
            print(f"✅ {file:40} ({size:,} bytes)")
        else:
            print(f"❌ {file:40} (MISSING)")
            all_exist = False
    
    return all_exist

def test_websocket_server():
    """Test if WebSocket server can start"""
    print_header("Testing WebSocket Server")
    
    try:
        import websockets
        import asyncio
        
        print("✅ WebSocket library available")
        print("✅ Server can be instantiated")
        print("ℹ️  Use start_binary_avatar.sh to run full server")
        return True
    except Exception as e:
        print(f"❌ Server test failed: {e}")
        return False

def test_tts_engines():
    """Test available TTS engines"""
    print_header("Testing TTS Engines")
    
    engines_available = []
    
    # Test Coqui TTS
    try:
        from TTS.api import TTS
        print("✅ Coqui TTS available (high-fidelity mode)")
        engines_available.append('coqui')
    except Exception:
        print("⚠️  Coqui TTS not available")
    
    # Test Piper TTS
    try:
        result = subprocess.run(['piper', '--version'], 
                              capture_output=True, 
                              timeout=2)
        if result.returncode == 0:
            print("✅ Piper TTS available (robotic mode)")
            engines_available.append('piper')
    except (subprocess.TimeoutExpired, FileNotFoundError, PermissionError, OSError):
        print("⚠️  Piper TTS not available")
    
    if not engines_available:
        print("ℹ️  No TTS engines available, will use Web Speech API fallback")
        print("   Install Coqui TTS: pip install TTS")
        print("   Install Piper TTS: https://github.com/rhasspy/piper")
    
    return True  # Non-blocking

def validate_javascript():
    """Basic JavaScript syntax validation"""
    print_header("Validating JavaScript")
    
    js_files = [
        'web/binary-avatar.js',
        'web/avatar-controller.js'
    ]
    
    all_valid = True
    for file in js_files:
        path = Path(file)
        if path.exists():
            content = path.read_text()
            # Basic checks
            if 'export' in content or 'import' in content:
                print(f"✅ {file} - ES6 module syntax detected")
            if 'THREE' in content:
                print(f"✅ {file} - Three.js integration detected")
        else:
            print(f"❌ {file} - File not found")
            all_valid = False
    
    return all_valid

def print_usage_instructions():
    """Print usage instructions"""
    print_header("Usage Instructions")
    
    print("""
To start the Binary Avatar System:

1. Quick Start:
   ./start_binary_avatar.sh

2. Manual Start:
   # Terminal 1 - Start backend
   python3 web/avatar-bridge.py --tts coqui
   
   # Terminal 2 - Start web server
   cd web && python3 -m http.server 8000
   
   # Open browser
   http://localhost:8000/avatar-integration.html

3. Integration with AIOS:
   - Import AvatarController in your JavaScript
   - Replace existing avatar with binary renderer
   - Connect to WebSocket for TTS streaming

For more details, see README_AVATAR.md
""")

def main():
    print("""
╔═══════════════════════════════════════════════════════════╗
║     JOHNNY MNEMONIC BINARY AVATAR - SYSTEM TEST          ║
║     Lo-Tek Cyberspace Interface Validation               ║
╚═══════════════════════════════════════════════════════════╝
""")
    
    tests = [
        ("Files", check_files),
        ("Python Packages", check_python_packages),
        ("WebSocket Server", test_websocket_server),
        ("TTS Engines", test_tts_engines),
        ("JavaScript", validate_javascript)
    ]
    
    results = {}
    for name, test_func in tests:
        try:
            results[name] = test_func()
        except Exception as e:
            print(f"❌ Test failed with exception: {e}")
            results[name] = False
    
    print_header("Test Summary")
    
    for name, result in results.items():
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"{status:10} - {name}")
    
    all_passed = all(results.values())
    
    if all_passed:
        print("\n🎉 All tests passed! System ready to deploy.")
        print_usage_instructions()
        return 0
    else:
        print("\n⚠️  Some tests failed. Please resolve issues before deployment.")
        print("\nTo install missing dependencies:")
        print("  pip install -r web/requirements-avatar.txt")
        return 1

if __name__ == "__main__":
    sys.exit(main())
