#!/bin/bash
# ╔═══════════════════════════════════════════════════════════════╗
# ║  AIOS — Johnny Mnemonic Binary Avatar System                ║
# ║  Lo-Tek Cyberspace Neural Interface                         ║
# ╚═══════════════════════════════════════════════════════════════╝
#
# Usage:
#   ./start_binary_avatar.sh           # Full Eel-based unified system
#   ./start_binary_avatar.sh --demo    # Standalone demo (no Eel required)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

MODE="unified"
if [ "$1" = "--demo" ]; then
    MODE="demo"
fi

echo ""
echo "  ╔═══════════════════════════════════════════════════════╗"
echo "  ║   █▀▀█ ▀█▀ █▀▀█ █▀▀   JOHNNY MNEMONIC AVATAR       ║"
echo "  ║   █▄▄█  █  █  █ ▀▀█   Lo-Tek Neural Interface       ║"
echo "  ║   █  █ ▄█▄ █▄▄█ ▀▀▀   Cyberspace Initializing...    ║"
echo "  ╚═══════════════════════════════════════════════════════╝"
echo ""

# ─── Dependency checks ───────────────────────────────────────────────
if ! command -v python3 &>/dev/null; then
    echo "  ✗ Python 3 not found"
    exit 1
fi

echo "  ▸ Checking dependencies..."

MISSING=()
for pkg in websockets numpy psutil; do
    if ! python3 -c "import $pkg" 2>/dev/null; then
        MISSING+=("$pkg")
    fi
done

if [ "$MODE" = "unified" ]; then
    for pkg in eel bottle; do
        if ! python3 -c "import $pkg" 2>/dev/null; then
            MISSING+=("$pkg")
        fi
    done
fi

if [ ${#MISSING[@]} -ne 0 ]; then
    echo "  ⚠ Missing: ${MISSING[*]}"
    echo "  ▸ Installing..."
    pip3 install "${MISSING[@]}" 2>/dev/null || pip install "${MISSING[@]}"
    echo ""
fi

# ─── TTS engine detection ────────────────────────────────────────────
TTS_ENGINE="fallback"
if python3 -c "from TTS.api import TTS" 2>/dev/null; then
    TTS_ENGINE="coqui"
    echo "  ✓ Coqui TTS — high-fidelity voice"
elif command -v piper &>/dev/null; then
    TTS_ENGINE="piper"
    echo "  ✓ Piper TTS — robotic voice"
else
    echo "  · No local TTS — using Web Speech API fallback"
fi

echo ""

# ─── Kill stale processes ────────────────────────────────────────────
pkill -f "avatar-bridge.py" 2>/dev/null || true
pkill -f "http.server.*8000" 2>/dev/null || true
sleep 0.5

# ═════════════════════════════════════════════════════════════════════
#  MODE: unified  — full Eel-based AIOS (recommended)
# ═════════════════════════════════════════════════════════════════════
if [ "$MODE" = "unified" ]; then
    echo "  ▸ Launching unified AIOS system..."
    echo "    Avatar bridge + Eel server on http://localhost:8000"
    echo ""
    echo "  ╔═══════════════════════════════════════════════════════╗"
    echo "  ║  NEURAL INTERFACE ONLINE                             ║"
    echo "  ║  http://localhost:8000                                ║"
    echo "  ║  TTS Engine: $TTS_ENGINE                                   ║"
    echo "  ║  Press Ctrl+C to terminate                           ║"
    echo "  ╚═══════════════════════════════════════════════════════╝"
    echo ""

    # run_onboarding.py starts the avatar bridge AND the Eel server
    exec python3 run_onboarding.py
fi

# ═════════════════════════════════════════════════════════════════════
#  MODE: demo  — standalone (no Eel, plain HTTP server)
# ═════════════════════════════════════════════════════════════════════
echo "  ▸ Starting standalone demo mode..."

# Start WebSocket backend
echo "  ▸ Avatar bridge → ws://localhost:8765"
python3 web/avatar-bridge.py --tts "$TTS_ENGINE" &
BACKEND_PID=$!
sleep 2

if ! ps -p $BACKEND_PID >/dev/null 2>&1; then
    echo "  ✗ Avatar bridge failed to start"
    exit 1
fi
echo "  ✓ Bridge PID $BACKEND_PID"

# Start static HTTP server
echo "  ▸ HTTP server  → http://localhost:8000"
cd web
python3 -m http.server 8000 --bind localhost &
WEB_PID=$!
cd "$SCRIPT_DIR"
sleep 1

if ! ps -p $WEB_PID >/dev/null 2>&1; then
    echo "  ✗ HTTP server failed"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi
echo "  ✓ HTTP PID $WEB_PID"

echo ""
echo "  ╔═══════════════════════════════════════════════════════╗"
echo "  ║  NEURAL INTERFACE ONLINE (DEMO)                      ║"
echo "  ║  http://localhost:8000/avatar-integration.html        ║"
echo "  ║  TTS: $TTS_ENGINE  |  WS: ws://localhost:8765              ║"
echo "  ║  Press Ctrl+C to terminate                           ║"
echo "  ╚═══════════════════════════════════════════════════════╝"
echo ""

# Open browser
if [[ "$OSTYPE" == "darwin"* ]]; then
    sleep 1 && open "http://localhost:8000/avatar-integration.html"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    sleep 1 && xdg-open "http://localhost:8000/avatar-integration.html" 2>/dev/null || true
fi

# Cleanup
cleanup() {
    echo ""
    echo "  ▸ Shutting down neural interface..."
    kill $BACKEND_PID 2>/dev/null || true
    kill $WEB_PID 2>/dev/null || true
    echo "  ✓ Shutdown complete"
    exit 0
}
trap cleanup INT TERM

wait
