# AI-OS Startup Script
# Executed automatically by init process (PID 1)

print("[STARTUP] Running AI-OS startup script...")

# Import kernel module
import aios_kernel
import sys
import os

print("[STARTUP] Kernel module loaded")

# --- Voice-guided onboarding (first boot) -----------------------------------
# Runs only once; subsequent boots skip via the flag file.
try:
    print("[STARTUP] Initializing voice onboarding assistant...")
    from kernel.voice_assistant import run_voice_onboarding, voice_command_repl
    skip_voice = os.environ.get("AIOS_VOICE_ONBOARDING", "1") == "0"
    if not skip_voice:
        run_voice_onboarding()
        # Keep a background voice command channel alive for the session
        voice_command_repl()
        print("[STARTUP] ✓ Voice assistant online")
    else:
        print("[STARTUP] Voice onboarding disabled by AIOS_VOICE_ONBOARDING=0")
except Exception as e:
    print(f"[STARTUP] Voice onboarding unavailable: {e}")

# Check hardware (optional, may not be wired yet)
print("[STARTUP] Hardware detection deferred (bindings pending)")

# Load AI-OS advanced layer
try:
    print("[STARTUP] Loading AI-OS advanced layer...")
    import aios_advanced_embedded
    
    # Create and start kernel
    kernel = aios_advanced_embedded.AdvancedAIOSKernel()
    kernel.start()
    
    print("[STARTUP] ✓ AI-OS advanced layer loaded")
    
    # Make available globally
    sys.modules['aios'] = kernel
    
except Exception as e:
    print(f"[STARTUP] Could not load advanced layer: {e}")
    print("[STARTUP] Falling back to basic Python shell")

# Minimal File I/O self-test
try:
    import os
    print("[STARTUP] File I/O self-test...")
    os.mkdir('/demo')
    fd = os.open('/demo/hello.txt', 'w')
    os.write(fd, b'Hello from AI-OS!')
    os.close(fd)
    print('[STARTUP] listdir(/demo):', os.listdir('/demo'))
    print('[STARTUP] stat(/demo/hello.txt):', os.stat('/demo/hello.txt'))
    os.remove('/demo/hello.txt')
    print('[STARTUP] after delete listdir(/demo):', os.listdir('/demo'))
except Exception as e:
    print('[STARTUP] File I/O self-test failed:', e)

print("[STARTUP] ✓ Startup complete\n")

# Show welcome message
print("╔════════════════════════════════════════════════════════════════╗")
print("║                                                                ║")
print("║            Welcome to AI-OS Python Shell                       ║")
print("║                                                                ║")
print("║  Your AI operating system is ready!                            ║")
print("║                                                                ║")
print("║  Try:                                                          ║")
print("║    help()        - Show available commands                     ║")
print("║    demo()        - Run AI-OS demo                              ║")
print("║    aios_kernel   - Access kernel functions                     ║")
print("║                                                                ║")
print("╚════════════════════════════════════════════════════════════════╝")
print()
