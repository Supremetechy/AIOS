For speech synthesis (TTS), install Coqui TTS:

```bash
pip install TTS
# On macOS, you may also need: brew install portaudio
# On Linux: sudo apt-get install libportaudio2


Coqui TTS will use your system's default audio player (afplay on macOS, aplay on Linux, or Windows Media.SoundPlayer). If you encounter issues, check your system audio setup.
## Requirements

Install dependencies:

```bash
pip install openai-whisper langchain openai tiktoken
```

You will also need ffmpeg for Whisper:

```bash
brew install ffmpeg  # macOS
# or
sudo apt-get install ffmpeg  # Linux
```

Set your OpenAI API key as an environment variable:

```bash
export OPENAI_API_KEY=sk-...
```
1. Replace the stub openai_llm_request/camera_vision_request callbacks with
     your actual API calls (SDKs or HTTP) so transcripts and vision insights
     flow directly from the real endpoints.
  2. Point streaming_callback/AdaptiveTTSService at live mic output and your
     speaker stack so sessions use genuine audio I/O.
  3. Plug actual LiDAR/camera/BLE readers into LiveSensor.poller and add
     further PolicyPlanner rules that codify your operational limits.

pip install scapy

sudo python3 main.py          # daemon mode

sudo python3 main.py --sniff --duration 60  # one-shot 60s capture