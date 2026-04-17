/**
 * Avatar Controller - Integrates Binary Avatar with TTS and Audio Analysis
 * Handles speech synthesis, audio analysis, and state management
 */

import { BinaryAvatarRenderer } from './binary-avatar.js';

class AvatarController {
  constructor(container, options = {}) {
    this.container = container;
    this.options = options;
    
    // Initialize binary avatar renderer
    this.renderer = new BinaryAvatarRenderer(container, {
      colorPalette: options.colorPalette || 'matrix',
      ...options
    });

    // Audio context for speech analysis
    this.audioContext = null;
    this.analyser = null;
    this.frequencyData = null;
    
    // Speech queue
    this.speechQueue = [];
    this.isSpeaking = false;
    
    // WebSocket for backend communication
    this.ws = null;
    this.wsReconnectAttempts = 0;
    this.maxReconnectAttempts = 5;

    // State
    this.currentEmotion = 'neutral';
    this.currentActivity = 'idle';
    
    this.init();
  }

  async init() {
    // Initialize audio context (requires user gesture)
    this.initAudioContext();
    
    // Connect to backend WebSocket
    this.connectWebSocket();
    
    // Start monitoring loop
    this.startMonitoring();
  }

  initAudioContext() {
    // Audio context must be created after user gesture
    // We'll initialize it on first interaction
    document.addEventListener('click', () => {
      if (!this.audioContext) {
        this.audioContext = new (window.AudioContext || window.webkitAudioContext)();
        this.analyser = this.audioContext.createAnalyser();
        this.analyser.fftSize = 256;
        this.frequencyData = new Uint8Array(this.analyser.frequencyBinCount);
        console.log('[Avatar] Audio context initialized');
      }
    }, { once: true });
  }

  connectWebSocket() {
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${wsProtocol}//${window.location.hostname}:8765`;

    try {
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        console.log('[Avatar] WebSocket connected to Piper TTS backend');
        this.wsReconnectAttempts = 0;
        this.renderer.setActivity('idle');
      };

      this.ws.onmessage = (event) => {
        this.handleWebSocketMessage(event.data);
      };

      this.ws.onerror = () => {
        // Silence repeated errors — onclose handles fallback
      };

      this.ws.onclose = () => {
        if (this.wsReconnectAttempts === 0) {
          console.log('[Avatar] Piper TTS server not running — using Web Speech API');
          this.useFallbackMode();
        } else {
          this.attemptReconnect();
        }
      };
    } catch (error) {
      this.useFallbackMode();
    }
  }

  attemptReconnect() {
    if (this.usingFallback) return;
    if (this.wsReconnectAttempts < this.maxReconnectAttempts) {
      this.wsReconnectAttempts++;
      const delay = Math.min(1000 * Math.pow(2, this.wsReconnectAttempts), 30000);
      setTimeout(() => this.connectWebSocket(), delay);
    } else {
      this.useFallbackMode();
    }
  }

  useFallbackMode() {
    if (this.usingFallback) return;
    this.usingFallback = true;
    this.ws = null;
    console.log('[Avatar] Using Web Speech API for voice output');
  }

  handleWebSocketMessage(data) {
    try {
      const message = JSON.parse(data);
      
      switch (message.type) {
        case 'audio_chunk':
          this.playAudioChunk(message.data);
          break;
        
        case 'state_update':
          this.updateState(message.state);
          break;
        
        case 'emotion':
          this.setEmotion(message.emotion);
          break;
        
        case 'system_status':
          this.updateSystemStatus(message.status);
          break;
        
        default:
          console.warn('[Avatar] Unknown message type:', message.type);
      }
    } catch (error) {
      console.error('[Avatar] Error parsing WebSocket message:', error);
    }
  }

  async speak(text, options = {}) {
    const emotion = options.emotion || 'neutral';
    
    // Add to speech queue
    this.speechQueue.push({ text, emotion, options });
    
    // Process queue if not already speaking
    if (!this.isSpeaking) {
      await this.processSpeechQueue();
    }
  }

  async processSpeechQueue() {
    if (this.speechQueue.length === 0) {
      this.isSpeaking = false;
      this.renderer.setActivity('idle');
      return;
    }

    this.isSpeaking = true;
    const { text, emotion, options } = this.speechQueue.shift();
    
    this.setEmotion(emotion);
    this.renderer.setActivity('speaking');

    try {
      if (this.ws && this.ws.readyState === WebSocket.OPEN && !this.usingFallback) {
        // Use backend TTS (Coqui or Piper)
        await this.speakViaBackend(text, emotion);
      } else {
        // Use Web Speech API fallback
        await this.speakViaWebSpeech(text);
      }
    } catch (error) {
      console.error('[Avatar] Speech error:', error);
    }

    // Process next item in queue
    await this.processSpeechQueue();
  }

  async speakViaBackend(text, emotion) {
    return new Promise((resolve, reject) => {
      // Send TTS request to backend
      this.ws.send(JSON.stringify({
        type: 'tts_request',
        text: text,
        emotion: emotion
      }));

      // Set up completion handler
      const completionHandler = (event) => {
        const message = JSON.parse(event.data);
        if (message.type === 'tts_complete') {
          this.ws.removeEventListener('message', completionHandler);
          resolve();
        }
      };

      this.ws.addEventListener('message', completionHandler);

      // Timeout after 30 seconds
      setTimeout(() => {
        this.ws.removeEventListener('message', completionHandler);
        reject(new Error('TTS timeout'));
      }, 30000);
    });
  }

  async speakViaWebSpeech(text) {
    return new Promise((resolve, reject) => {
      if (!window.speechSynthesis) {
        reject(new Error('Speech synthesis not supported'));
        return;
      }

      const utterance = new SpeechSynthesisUtterance(text);
      
      // Configure voice
      const voices = window.speechSynthesis.getVoices();
      const preferredVoice = voices.find(v => v.lang.startsWith('en'));
      if (preferredVoice) {
        utterance.voice = preferredVoice;
      }
      
      utterance.rate = 1.0;
      utterance.pitch = 1.0;
      utterance.volume = 1.0;

      // Analyze audio during speech
      utterance.onstart = () => {
        this.startAudioAnalysis();
      };

      utterance.onend = () => {
        this.stopAudioAnalysis();
        resolve();
      };

      utterance.onerror = (event) => {
        this.stopAudioAnalysis();
        reject(event);
      };

      window.speechSynthesis.speak(utterance);
    });
  }

  playAudioChunk(audioData) {
    if (!this.audioContext) return;

    // Decode and play audio chunk from backend
    const audioBuffer = this.base64ToArrayBuffer(audioData);
    
    this.audioContext.decodeAudioData(audioBuffer, (buffer) => {
      const source = this.audioContext.createBufferSource();
      source.buffer = buffer;
      
      // Connect to analyser for visualization
      source.connect(this.analyser);
      this.analyser.connect(this.audioContext.destination);
      
      source.start(0);
      
      // Start audio-reactive animation
      if (!this.isAnalyzing) {
        this.startAudioAnalysis();
      }
    });
  }

  startAudioAnalysis() {
    if (this.isAnalyzing || !this.analyser) return;
    
    this.isAnalyzing = true;
    
    const analyze = () => {
      if (!this.isAnalyzing) return;
      
      // Get frequency data
      this.analyser.getByteFrequencyData(this.frequencyData);
      
      // Calculate average volume
      let sum = 0;
      for (let i = 0; i < this.frequencyData.length; i++) {
        sum += this.frequencyData[i];
      }
      const avgVolume = sum / this.frequencyData.length / 255;
      
      // Update avatar
      this.renderer.setVolume(avgVolume);
      this.renderer.setFrequencyData(this.frequencyData);
      
      requestAnimationFrame(analyze);
    };
    
    analyze();
  }

  stopAudioAnalysis() {
    this.isAnalyzing = false;
    this.renderer.setVolume(0);
  }

  setEmotion(emotion) {
    this.currentEmotion = emotion;
    this.renderer.setEmotion(emotion);
    
    // Optionally change color palette based on emotion
    const emotionPalettes = {
      'happy': 'cyan',
      'neutral': 'matrix',
      'thinking': 'amber',
      'error': 'red',
      'excited': 'cyber-magenta'
    };
    
    const palette = emotionPalettes[emotion];
    if (palette) {
      this.renderer.options.colorPalette = palette;
      this.renderer.digitMesh.material.uniforms.uColorPalette.value = this.renderer.getColorPalette();
    }
  }

  updateState(state) {
    if (state.activity) {
      this.renderer.setActivity(state.activity);
    }
    if (state.emotion) {
      this.setEmotion(state.emotion);
    }
    if (state.cpuLoad !== undefined) {
      this.renderer.setState({ cpuLoad: state.cpuLoad });
    }
  }

  updateSystemStatus(status) {
    // Update visualization based on system status
    if (status.cpu_usage > 80) {
      this.renderer.setActivity('thinking');
    }
  }

  startMonitoring() {
    // Periodic state updates (even without WebSocket)
    setInterval(() => {
      // Request system stats if WebSocket is available
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({
          type: 'request_status'
        }));
      }
    }, 5000);
  }

  stop() {
    // Stop current speech
    this.speechQueue = [];
    this.isSpeaking = false;
    this.stopAudioAnalysis();
    
    if (window.speechSynthesis) {
      window.speechSynthesis.cancel();
    }
    
    this.renderer.setActivity('idle');
  }

  // Utility methods
  base64ToArrayBuffer(base64) {
    const binaryString = window.atob(base64);
    const len = binaryString.length;
    const bytes = new Uint8Array(len);
    for (let i = 0; i < len; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes.buffer;
  }

  destroy() {
    this.stop();
    
    if (this.ws) {
      this.ws.close();
    }
    
    if (this.audioContext) {
      this.audioContext.close();
    }
    
    this.renderer.destroy();
  }
}

// Export for use in other modules
export { AvatarController };

// Global API for backward compatibility
if (typeof window !== 'undefined') {
  window.AIOS = window.AIOS || {};
  window.AIOS.AvatarController = AvatarController;
}
