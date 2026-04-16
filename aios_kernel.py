from kernel.compatibility_checker import CompatibilityChecker
#!/usr/bin/env python3
"""
AI-OS Kernel - Main Entry Point
A standalone operating system designed for AI-based workloads
"""

import sys
import argparse
import os
import importlib
import tempfile
try:
    from TTS.api import TTS as CoquiTTS
except ImportError:
    CoquiTTS = None
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import Any, Callable, Dict, Iterable, List, Optional, Tuple
import whisper  # OpenAI Whisper for STT
from langchain_openai import ChatOpenAI
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.output_parsers import StrOutputParser

from kernel import (
    HardwareDetector,
    SystemSpecs,
    BootLoader,
    BootConfig,
    FileSystemManager,
    NetworkManager,
    SystemMonitor,
    ContainerManager,
    ModelManager,
    JobScheduler,
    ResourceManager,
    DistributedCoordinator,
    ResourcePool,
    ResourceRequirements,
    ResourceQuota,
    GPUScheduler,
    PowerPolicy,
    TenantRegistry,

)

def _voice_group_enabled() -> bool:
    groups = os.environ.get("AIOS_GROUPS", "")  # type: ignore[attr-defined]
    return "voice" in {g.strip().lower() for g in groups.split(",") if g.strip()}


def _sync_groups_to_kernel() -> None:
    groups = os.environ.get("AIOS_GROUPS", "")
    if not groups:
        return
    try:
        embedded = importlib.import_module("python_embed.aios_kernel")
        if hasattr(embedded, "set_groups"):
            embedded.set_groups(groups)
    except (ImportError, AttributeError) as err:
        print(f"[AIOS] Group sync error: {err}")


_sync_groups_to_kernel()


class RealWorldItem:
    def __init__(
        self,
        item_id: Optional[str] = None,
        name: str = "",
        description: str = "",
        location: Tuple[float, float, float] = (0.0, 0.0, 0.0),
        metadata: Optional[Dict[str, Any]] = None,
        id: Optional[str] = None,
    ):
        if item_id is None:
            item_id = id or ""
        self.item_id = item_id
        self.name = name
        self.description = description
        self.location = location
        self.metadata = metadata or {}

    @property
    def id(self) -> str:
        return self.item_id

class SensorUpdate:
    def __init__(self, source: str, timestamp: datetime, payload: Dict[str, Any]):
        self.source = source
        self.timestamp = timestamp
        self.payload = payload

class PhysicalContext:
    def __init__(self) -> None:
        self._items: Dict[str, RealWorldItem] = {}
    def add_item(self, item: RealWorldItem) -> None:
        self._items[item.item_id] = item
    def describe(self) -> str:
        if not self._items:
            return "No registered items in the environment."
        fragments = [f"{item.name} at {item.location} ({item.description})" for item in self._items.values()]
        return "; ".join(fragments)
    def find_nearest(self, point: Tuple[float, float, float]) -> Optional[RealWorldItem]:
        best_item = None
        best_dist = float("inf")
        for item in self._items.values():
            dx = item.location[0] - point[0]
            dy = item.location[1] - point[1]
            dz = item.location[2] - point[2]
            dist = (dx ** 2 + dy ** 2 + dz ** 2) ** 0.5
            if dist < best_dist:
                best_dist = dist
                best_item = item
        return best_item
    def list_item_names(self) -> List[str]:
        return [item.name for item in self._items.values()]
    def ingest_sensor_update(self, update: SensorUpdate) -> None:
        for raw_item in update.payload.get("items", []):
            item = RealWorldItem(
                item_id=raw_item.get("id", f"req-{len(self._items)+1}"),
                name=raw_item["name"],
                description=raw_item.get("description", ""),
                location=tuple(raw_item.get("location", (0.0, 0.0, 0.0))),
                metadata={
                    "source": update.source,
                    "timestamp": update.timestamp.isoformat(),
                    **raw_item.get("metadata", {}),
                },
            )
            self.add_item(item)

AudioStream = Iterable[bytes]
StreamingCallback = Callable[[AudioStream, Dict[str, Any]], str]
AdaptiveAudioGenerator = Callable[[str], bytes]
PlaybackHandler = Callable[[bytes], None]

LLMRequest = Callable[[str, str, Dict[str, Any]], Dict[str, Any]]
VisionRequest = Callable[[str, str, Dict[str, Any]], Dict[str, Any]]

class ASRService(ABC):
    @abstractmethod
    def transcribe(self, audio_stream: Iterable[bytes], metadata: Dict[str, Any]) -> str:
        ...

class WhisperASRService(ASRService):
    def __init__(self, model_name: str = "base"):
        self.model = whisper.load_model(model_name)
    def transcribe(self, audio_stream: Iterable[bytes], metadata: Dict[str, Any]) -> str:
        if not _voice_group_enabled():
            raise PermissionError("Voice group required for audio capture/transcription")
        audio_bytes = b"".join(audio_stream)
        with tempfile.NamedTemporaryFile(suffix=".wav", delete=True) as f:
            f.write(audio_bytes)
            f.flush()
            result = self.model.transcribe(f.name, language=metadata.get("language", "en"))
        return result["text"]

class TTSService(ABC):
    @abstractmethod
    def speak(self, text: str) -> None:
        ...

class DummyASRService(ASRService):
    def __init__(self, preset: str) -> None:
        self.preset = preset
    def transcribe(self, audio_stream: Iterable[bytes], metadata: Dict[str, Any]) -> str:
        return self.preset

class DummyTTSService(TTSService):
    def speak(self, text: str) -> None:
        print(f"[TTS] {text}")

class CoquiTTService(TTSService):
    def __init__(self, model_name: str = "tts_models/en/ljspeech/tacotron2-DDC", vocoder_name: Optional[str] = None):
        if CoquiTTS is None:
            raise ImportError("Coqui TTS is not installed. Please install with 'pip install TTS'.")
        self.tts = CoquiTTS(model_name)
        self.vocoder_name = vocoder_name
    def speak(self, text: str) -> None:
        if not _voice_group_enabled():
            raise PermissionError("Voice group required for TTS playback")
        with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as f:
            self.tts.tts_to_file(text=text, file_path=f.name)
            f.flush()
            import platform
            import subprocess
            if platform.system() == "Darwin":
                subprocess.run(["afplay", f.name], check=True)
            elif platform.system() == "Linux":
                subprocess.run(["aplay", f.name], check=True)
            elif platform.system() == "Windows":
                subprocess.run(["powershell", "-c", f"(New-Object Media.SoundPlayer '{f.name}').PlaySync();"], check=True)
            else:
                print(f"[TTS] Audio saved to {f.name}, please play manually.")

class VoiceCommunicator:
    def __init__(self, asr: ASRService, tts: TTSService) -> None:
        self.asr = asr
        self.tts = tts
    def listen(self, audio: Optional[bytes] = None, audio_stream: Optional[Iterable[bytes]] = None, metadata: Optional[Dict[str, Any]] = None) -> str:
        metadata = metadata or {}
        stream = audio_stream
        if stream is None:
            stream = [audio or b""]
        return self.asr.transcribe(stream, metadata)
    def speak(self, message: str) -> None:
        self.tts.speak(message)

class StreamingASRService(ASRService):
    """Wraps any streaming-capable ASR provider via callback injection."""
    def __init__(self, stream_callback: StreamingCallback) -> None:
        self._stream_callback = stream_callback
    def transcribe(self, audio_stream: Iterable[bytes], metadata: Dict[str, Any]) -> str:
        return self._stream_callback(audio_stream, metadata)

class AdaptiveTTSService(TTSService):
    """Calls a streaming/adaptive TTS provider and hands the audio bytes to a playback handler."""
    def __init__(self, generator: AdaptiveAudioGenerator, playback: PlaybackHandler) -> None:
        self._generator = generator
        self._playback = playback
    def speak(self, text: str) -> None:
        audio = self._generator(text)
        self._playback(audio)

def file_based_playback(audio: bytes, path: str = "tts_output.raw") -> None:
    from pathlib import Path
    Path(path).write_bytes(audio)
    print(f"[TTS] persisted {len(audio)} bytes to {path}")

def iter_audio_file(path: str, chunk_size: int = 4096) -> Iterable[bytes]:
    from pathlib import Path
    file_path = Path(path)
    if not file_path.exists():
        raise FileNotFoundError(f"{path} not found")
    with file_path.open("rb") as fp:
        while True:
            chunk = fp.read(chunk_size)
            if not chunk:
                break
            yield chunk

class SpatialSensor(ABC):
    @abstractmethod
    def poll(self) -> List[SensorUpdate]:
        ...

class StaticSpatialSensor(SpatialSensor):
    """Simulates spatial sensor output from a static JSON payload."""
    def __init__(self, updates: List[SensorUpdate]) -> None:
        self._updates = updates
        self._index = 0
    def poll(self) -> List[SensorUpdate]:
        if self._index >= len(self._updates):
            return []
        update = self._updates[self._index]
        self._index += 1
        return [update]

class LiveSensor(SpatialSensor):
    """Polls an arbitrary live source through a provided callable."""
    def __init__(self, poller: Callable[[], List[SensorUpdate]]) -> None:
        self._poller = poller
    def poll(self) -> List[SensorUpdate]:
        return self._poller()

class BaseModelAdapter(ABC):
    def __init__(self, model_name: str, model_type: str):
        self.model_name = model_name
        self.model_type = model_type
    @abstractmethod
    def interpret(self, prompt: str, context: str) -> Dict[str, Any]:
        ...

class LLMAdapter(BaseModelAdapter):
    """Stub for a large language model adapter (hook to real APIs here)."""
    def interpret(self, prompt: str, context: str) -> Dict[str, Any]:
        summary = (
            f"[LLM:{self.model_name}] interpreted '{prompt}' with context "
            f"'{context[:80]}'"
        )
        return {"model": self.model_name, "type": self.model_type, "summary": summary, "confidence": 0.92}

class VisionAdapter(BaseModelAdapter):
    """Stub for a vision or multi-modal model adapter."""
    def interpret(self, prompt: str, context: str) -> Dict[str, Any]:
        summary = (
            f"[Vision:{self.model_name}] mapped imagery over '{context[:60]}' for prompt '{prompt}'"
        )
        return {"model": self.model_name, "type": self.model_type, "summary": summary, "confidence": 0.85}

class ExternalLLMAdapter(BaseModelAdapter):
    """Connects to an external LLM endpoint via a request callback."""
    def __init__(self, model_name: str, request_fn: LLMRequest) -> None:
        super().__init__(model_name, "llm")
        self._request_fn = request_fn
    def interpret(self, prompt: str, context: str) -> Dict[str, Any]:
        payload = self._request_fn(prompt, context, {"model": self.model_name})
        return {
            "model": self.model_name,
            "type": self.model_type,
            "summary": payload.get("summary", ""),
            "confidence": payload.get("confidence", 0.5),
            "details": payload.get("details"),
        }

class ExternalVisionAdapter(BaseModelAdapter):
    """Wraps an external vision endpoint and returns the enriched metadata."""
    def __init__(self, model_name: str, request_fn: VisionRequest) -> None:
        super().__init__(model_name, "vision")
        self._request_fn = request_fn
    def interpret(self, prompt: str, context: str) -> Dict[str, Any]:
        payload = self._request_fn(prompt, context, {"model": self.model_name})
        return {
            "model": self.model_name,
            "type": self.model_type,
            "summary": payload.get("summary", ""),
            "confidence": payload.get("confidence", 0.5),
            "vision_tags": payload.get("vision_tags", []),
            "insights": payload.get("insights"),
        }

class LangChainLLMAdapter(BaseModelAdapter):
    def __init__(self, model_name: str, openai_api_key: str):
        super().__init__(model_name, "llm")
        # Wrap api_key in a lambda for compatibility with SecretStr | Callable
        self.llm = ChatOpenAI(model=model_name, api_key=(lambda: openai_api_key))
        self.prompt_template = ChatPromptTemplate.from_template(
            """Context: {context}\nPrompt: {prompt}\nAnswer:"""
        )
        self.output_parser = StrOutputParser()
    def interpret(self, prompt: str, context: str) -> Dict[str, Any]:
        chain = self.prompt_template | self.llm | self.output_parser
        result = chain.invoke({"prompt": prompt, "context": context})
        return {
            "model": self.model_name,
            "type": self.model_type,
            "summary": result,
            "trace": None,
            "confidence": 0.95
        }

class MultiModelCoordinator:
    def __init__(self, adapters: List[BaseModelAdapter]) -> None:
        self.adapters = adapters
    def query(self, prompt: str, context: str) -> Dict[str, Any]:
        responses = [adapter.interpret(prompt, context) for adapter in self.adapters]
        merged_summary = " | ".join(response["summary"] for response in responses)
        return {"responses": responses, "merged_summary": merged_summary}

PolicyAction = Dict[str, Any]
PolicyRule = Callable[[Dict[str, Any], PhysicalContext], Optional[PolicyAction]]

class PolicyPlanner:
    def __init__(self, rules: List[PolicyRule]):
        self.rules = rules
    def decide(self, inference: Dict[str, Any], context: PhysicalContext) -> Dict[str, Any]:
        actions = []
        for rule in self.rules:
            decision = rule(inference, context)
            if decision:
                actions.append(decision)
        return {"actions": actions}

class AIOS:
    def __init__(self, communicator: VoiceCommunicator, context: PhysicalContext, coordinator: MultiModelCoordinator, planner: PolicyPlanner, sensors: Optional[List[Any]] = None):
        self.communicator = communicator
        self.context = context
        self.coordinator = coordinator
        self.planner = planner
        self.sensors = sensors or []
    def register_sensor(self, sensor: SpatialSensor) -> None:
        self.sensors.append(sensor)
    def _compile_context(self) -> str:
        return f"Context: {self.context.describe()}"
    def _ingest_sensor_updates(self) -> None:
        for sensor in self.sensors:
            for update in sensor.poll():
                self.context.ingest_sensor_update(update)
    def handle_voice_session(self, audio: Optional[bytes] = None, audio_stream: Optional[Iterable[bytes]] = None, metadata: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        try:
            self._ingest_sensor_updates()
            transcript = self.communicator.listen(audio=audio, audio_stream=audio_stream, metadata=metadata or {})
            physical_blob = self._compile_context()
            inference = self.coordinator.query(transcript, physical_blob)
            policy = self.planner.decide(inference, self.context)
            response = f"I heard '{transcript}'. Scene: {physical_blob}. {inference['merged_summary']}"
            if policy["actions"]:
                response += " Actions: " + ", ".join(act.get("label", "unnamed") for act in policy["actions"])
            self.communicator.speak(response)
            return {"inference": inference, "policy": policy}
        except (RuntimeError, ValueError, PermissionError) as err:
            print(f"[AIOS] Error in handle_voice_session: {err}")
            return {"error": str(err)}

    def continuous_listen_and_respond(self, interval_seconds: int = 0, schedule: Optional[List[int]] = None):
        """Continuously listen and respond to spoken commands. Optionally schedule sessions."""
        import threading
        import time
        def listen_loop():
            print("[AIOS] Continuous voice session started. Say 'exit' to stop.")
            while True:
                try:
                    print("[AIOS] Listening for command...")
                    result = self.handle_voice_session(metadata={"language": "en-US", "location": "continuous"})
                    transcript = ''
                    if 'inference' in result and result['inference'].get('responses'):
                        transcript = result['inference']['responses'][0].get('summary', '')
                    if transcript and transcript.strip().lower() == 'exit':
                        print("[AIOS] Voice session ended by user.")
                        break
                    if interval_seconds > 0:
                        time.sleep(interval_seconds)
                except (RuntimeError, ValueError, PermissionError) as err:
                    print(f"[AIOS] Voice session error: {err}")
                    time.sleep(2)
        if schedule:
            import sched
            scheduler = sched.scheduler(time.time, time.sleep)
            now = datetime.now()
            for s in schedule:
                run_at = now.replace(second=0, microsecond=0) + timedelta(seconds=s)
                delay = (run_at - now).total_seconds()
                scheduler.enter(delay, 1, listen_loop)
            threading.Thread(target=scheduler.run, daemon=True).start()
        else:
            threading.Thread(target=listen_loop, daemon=True).start()

class AIOSKernel:
    """Main AI-OS Kernel"""
    
    def __init__(self, boot_config: Optional[BootConfig] = None):
        self.boot_config = boot_config or BootConfig()
        self.system_specs: Optional[SystemSpecs] = None
        self.fs_manager: Optional[FileSystemManager] = None
        self.net_manager: Optional[NetworkManager] = None
        self.monitor: Optional[SystemMonitor] = None
        self.running = False
        self.aios: Optional[AIOS] = None  # Ensure aios is always defined
        
    def start(self) -> bool:
        """Start the AI-OS kernel"""
        print("\n" + "╔" + "═"*78 + "╗")
        print("║" + " "*78 + "║")
        print("║" + "AI-OS - Operating System for AI Workloads".center(78) + "║")
        print("║" + "Initializing Kernel...".center(78) + "║")
        print("║" + " "*78 + "║")
        print("╚" + "═"*78 + "╝\n")
        
        # Phase 1: Boot
        if not self._boot_system():
            print("[KERNEL] Boot failed. Exiting.")
            return False
        
        # Phase 2: Initialize subsystems
        if not self._initialize_subsystems():
            print("[KERNEL] Subsystem initialization failed. Exiting.")
            return False
        
        # Phase 3: Post-boot diagnostics
        self._run_diagnostics()
        
        # Phase 4: Ready
        self._kernel_ready()
        
        self.running = True
        return True
    
    def _boot_system(self) -> bool:
        """Boot the system"""
        print("[KERNEL] Phase 1: System Boot")
        print("-" * 80)
        
        bootloader = BootLoader(self.boot_config)
        success = bootloader.boot()
        
        if success:
            self.system_specs = bootloader.system_specs
            print("[KERNEL] ✓ Boot completed successfully\n")
        else:
            print("[KERNEL] ✗ Boot failed\n")
        
        return success
    
    def _initialize_subsystems(self) -> bool:
        """Initialize kernel subsystems"""
        print("[KERNEL] Phase 2: Subsystem Initialization")
        print("-" * 80)
        
        try:
            # Initialize filesystem manager
            print("[KERNEL] Initializing filesystem manager...")
            self.fs_manager = FileSystemManager()
            self.fs_manager.initialize()
            
            # Initialize network manager
            print("[KERNEL] Initializing network manager...")
            self.net_manager = NetworkManager()
            self.net_manager.initialize()
            
            # Initialize system monitor
            print("[KERNEL] Initializing system monitor...")
            self.monitor = SystemMonitor()
            
            print("[KERNEL] ✓ All subsystems initialized\n")
            return True
            
        except (RuntimeError, ValueError, PermissionError) as err:
            print(f"[KERNEL] ✗ Subsystem initialization failed: {err}\n")
            return False
    
    def _run_diagnostics(self):
        """Run post-boot diagnostics"""
        print("[KERNEL] Phase 3: System Diagnostics")
        print("-" * 80)
        
        # Get system snapshot
        if self.monitor:
            snapshot = self.monitor.get_snapshot()
            
            # Check system health
            warnings = []
            
            # CPU check
            if snapshot.cpu_stats.usage_percent > 90:
                warnings.append("⚠ High CPU usage detected")
            
            # Memory check
            if snapshot.memory_stats.usage_percent > 90:
                warnings.append("⚠ Low memory available")
            
            # GPU check
            for gpu in snapshot.gpu_stats:
                if gpu.memory_percent > 90:
                    warnings.append(f"⚠ GPU {gpu.gpu_id} memory almost full")
                if gpu.temperature_celsius and gpu.temperature_celsius > 85:
                    warnings.append(f"⚠ GPU {gpu.gpu_id} temperature high ({gpu.temperature_celsius:.1f}°C)")
            
            # Disk check
            for disk in snapshot.disk_stats:
                if disk.usage_percent > 90:
                    warnings.append(f"⚠ Disk {disk.mount_point} almost full ({disk.usage_percent:.1f}%)")
            
            if warnings:
                print("[KERNEL] System Warnings:")
                for warning in warnings:
                    print(f"[KERNEL]   {warning}")
            else:
                print("[KERNEL] ✓ All system checks passed")
            
            print()
    
    def _kernel_ready(self):
        """Kernel is ready"""
        print("[KERNEL] Phase 4: System Ready")
        print("-" * 80)
        print("[KERNEL] ✓ AI-OS Kernel is ready")
        print("[KERNEL] ✓ All systems operational\n")
        self._print_system_summary()
        # --- Voice-guided onboarding (first boot) + always-on voice channel ---
        try:
            from kernel.voice_assistant import (
                run_voice_onboarding,
                VoiceOnboardingAssistant,
            )
            if os.environ.get("AIOS_VOICE_ONBOARDING", "1") != "0":
                print("[KERNEL] Launching voice-guided onboarding (first boot only)...")
                run_voice_onboarding()
                self.voice_assistant = VoiceOnboardingAssistant()
                self.voice_assistant.start_always_on()
                print("[KERNEL] ✓ Always-on voice command channel online")
        except Exception as err:
            print(f"[KERNEL] Voice onboarding unavailable: {err}")
        # --- Automatically run continuous AI/voice session after boot ---
        try:
            print("[KERNEL] Launching always-on AI/voice session (continuous listening)...")
            if self.aios:
                self.aios.continuous_listen_and_respond(interval_seconds=0)
        except (RuntimeError, ValueError, PermissionError) as err:
            print(f"[KERNEL] AI/voice session failed to start: {err}")
    
    def _print_system_summary(self):
        """Print system summary"""
        print("\n" + "╔" + "═"*78 + "╗")
        print("║" + " SYSTEM SUMMARY ".center(78, "═") + "║")
        print("╚" + "═"*78 + "╝\n")
        
        if self.system_specs:
            # Processors
            print("COMPUTE RESOURCES:")
            print("-" * 80)
            
            cpu_count = sum(1 for p in self.system_specs.processors if p.processor_type.value == 'cpu')
            gpu_count = sum(1 for p in self.system_specs.processors if p.processor_type.value == 'gpu')
            tpu_count = sum(1 for p in self.system_specs.processors if p.processor_type.value == 'tpu')
            
            print(f"  CPUs: {cpu_count}")
            for proc in self.system_specs.processors:
                if proc.processor_type.value == 'cpu':
                    print(f"    • {proc.vendor.value.upper()} {proc.model}")
                    print(f"      {proc.cores} cores, {proc.threads} threads @ {proc.frequency_mhz:.0f} MHz")
            
            if gpu_count > 0:
                print(f"\n  GPUs: {gpu_count}")
                for proc in self.system_specs.processors:
                    if proc.processor_type.value == 'gpu':
                        print(f"    • {proc.vendor.value.upper()} {proc.model}")
                        if proc.memory_gb:
                            print(f"      VRAM: {proc.memory_gb:.2f} GB")
                        if proc.compute_capability:
                            print(f"      Compute: {proc.compute_capability}")
            
            if tpu_count > 0:
                print(f"\n  TPUs: {tpu_count}")
                for proc in self.system_specs.processors:
                    if proc.processor_type.value == 'tpu':
                        print(f"    • {proc.vendor.value.upper()} {proc.model}")
            
            # Memory
            print("\n\nMEMORY:")
            print("-" * 80)
            print(f"  System RAM: {self.system_specs.memory.total_gb:.2f} GB")
            print(f"  Available: {self.system_specs.memory.available_gb:.2f} GB")
            
            total_vram = sum(p.memory_gb for p in self.system_specs.processors 
                           if p.memory_gb is not None)
            if total_vram > 0:
                print(f"  GPU VRAM: {total_vram:.2f} GB")
            
            # Storage
            print("\n\nSTORAGE:")
            print("-" * 80)
            total_storage = sum(d.total_gb for d in self.system_specs.storage_devices)
            print(f"  Total: {total_storage:.2f} GB across {len(self.system_specs.storage_devices)} device(s)")
            
            # Network
            print("\n\nNETWORK:")
            print("-" * 80)
            print(f"  Interfaces: {len(self.system_specs.network_interfaces)}")
            
            if self.net_manager:
                default_iface = self.net_manager.get_default_interface()
                if default_iface:
                    print(f"  Default: {default_iface}")
        
        print("\n" + "═"*80)
        print("\n[KERNEL] Type 'help' for available commands")
        print("[KERNEL] Type 'monitor' to start system monitoring")
        print("[KERNEL] Type 'exit' to shutdown\n")
    
    def run_interactive_shell(self):
        """Run interactive kernel shell"""
        print("[KERNEL] Starting interactive shell...\n")
        
        while self.running:
            try:
                command = input("aios> ").strip().lower()
                
                if not command:
                    continue
                
                if command == 'exit' or command == 'quit':
                    self._shutdown()
                    break
                
                elif command == 'help':
                    self._show_help()
                
                elif command == 'status':
                    self._show_status()
                
                elif command == 'monitor':
                    self._start_monitoring()
                
                elif command == 'hardware':
                    self._show_hardware()
                
                elif command == 'network':
                    self._show_network()
                
                elif command == 'filesystem' or command == 'fs':
                    self._show_filesystem()
                
                elif command == 'specs':
                    if self.system_specs:
                        HardwareDetector().print_system_report(self.system_specs)
                
                else:
                    print(f"Unknown command: {command}. Type 'help' for available commands.")
                
            except KeyboardInterrupt:
                print("\n[KERNEL] Use 'exit' to shutdown")
            except (RuntimeError, ValueError, PermissionError) as err:
                print(f"[KERNEL] Error: {err}")
    
    def _show_help(self):
        """Show help"""
        print("\nAvailable Commands:")
        print("-" * 80)
        print("  help        - Show this help message")
        print("  status      - Show current system status")
        print("  monitor     - Start system monitoring")
        print("  hardware    - Show hardware information")
        print("  network     - Show network configuration")
        print("  filesystem  - Show filesystem information")
        print("  specs       - Show detailed system specifications")
        print("  exit        - Shutdown the system")
        print()
    
    def _show_status(self):
        """Show system status"""
        if self.monitor:
            snapshot = self.monitor.get_snapshot()
            self.monitor.print_snapshot(snapshot)
    
    def _start_monitoring(self):
        """Start system monitoring"""
        print("[KERNEL] Starting monitoring (press Ctrl+C to stop)...")
        try:
            if self.monitor:
                self.monitor.monitor_continuous(interval_seconds=5, duration_seconds=30)
        except KeyboardInterrupt:
            print("\n[KERNEL] Monitoring stopped")
    
    def _show_hardware(self):
        """Show hardware info"""
        if self.system_specs:
            print("\nHardware Summary:")
            print("-" * 80)
            for proc in self.system_specs.processors:
                print(f"{proc.processor_type.value.upper()}: {proc.vendor.value} {proc.model}")
    
    def _show_network(self):
        """Show network info"""
        if self.net_manager:
            self.net_manager.print_network_summary()
    
    def _show_filesystem(self):
        """Show filesystem info"""
        if self.fs_manager:
            self.fs_manager.print_mount_summary()
    
    def _shutdown(self):
        """Shutdown the system"""
        print("\n[KERNEL] Initiating shutdown...")
        print("[KERNEL] Stopping services...")
        print("[KERNEL] Unmounting filesystems...")
        print("[KERNEL] Powering down...")
        print("[KERNEL] ✓ System halted\n")
        self.running = False


class AdvancedAIOSKernel:
    """Advanced AI-OS Kernel with enterprise features"""

    def __init__(self, boot_config: Optional[BootConfig] = None):
        self.compat_checker = CompatibilityChecker()
        self.boot_config = boot_config or BootConfig()
        self.system_specs: Optional[SystemSpecs] = None
        
        # Core subsystems
        self.fs_manager: Optional[FileSystemManager] = None
        self.net_manager: Optional[NetworkManager] = None
        self.monitor: Optional[SystemMonitor] = None
        
        # Advanced subsystems
        self.container_manager: Optional[ContainerManager] = None
        self.model_manager: Optional[ModelManager] = None
        self.resource_manager: Optional[ResourceManager] = None
        self.scheduler: Optional[JobScheduler] = None
        self.distributed_coordinator: Optional[DistributedCoordinator] = None
        
        self.running = False
        
    def start(self) -> bool:
        """Start the advanced AI-OS kernel"""
        print("\n" + "╔" + "═"*78 + "╗")
        print("║" + " "*78 + "║")
        print("║" + "AI-OS ADVANCED - Enterprise AI Operating System".center(78) + "║")
        print("║" + "With Container, Model Management & Distributed Training".center(78) + "║")
        print("║" + " "*78 + "║")
        print("╚" + "═"*78 + "╝\n")
        
        # Phase 1: Boot
        if not self._boot_system():
            print("[KERNEL] Boot failed. Exiting.")
            return False
        
        # Phase 2: Initialize core subsystems
        if not self._initialize_core_subsystems():
            print("[KERNEL] Core subsystem initialization failed. Exiting.")
            return False
        
        # Phase 3: Initialize advanced subsystems
        if not self._initialize_advanced_subsystems():
            print("[KERNEL] Advanced subsystem initialization failed (non-critical).")
        
        # Phase 4: Post-boot diagnostics
        self._run_diagnostics()
        
        # Phase 5: Ready
        self._kernel_ready()
        
        self.running = True
        return True
    
    def _boot_system(self) -> bool:
        """Boot the system"""
        print("[KERNEL] Phase 1: System Boot")
        print("-" * 80)
        
        bootloader = BootLoader(self.boot_config)
        success = bootloader.boot()
        
        if success:
            self.system_specs = bootloader.system_specs
            print("[KERNEL] ✓ Boot completed successfully\n")
        else:
            print("[KERNEL] ✗ Boot failed\n")
        
        return success
    
    def _initialize_core_subsystems(self) -> bool:
        """Initialize core kernel subsystems"""
        print("[KERNEL] Phase 2: Core Subsystem Initialization")
        print("-" * 80)
        
        try:
            # Filesystem
            print("[KERNEL] Initializing filesystem manager...")
            self.fs_manager = FileSystemManager()
            self.fs_manager.initialize()
            
            # Network
            print("[KERNEL] Initializing network manager...")
            self.net_manager = NetworkManager()
            self.net_manager.initialize()
            
            # Monitor
            print("[KERNEL] Initializing system monitor...")
            self.monitor = SystemMonitor()
            
            print("[KERNEL] ✓ Core subsystems initialized\n")
            return True
            
        except Exception as e:
            print(f"[KERNEL] ✗ Core subsystem initialization failed: {e}\n")
            return False
    
    def _initialize_advanced_subsystems(self) -> bool:
        """Initialize advanced subsystems"""
        print("[KERNEL] Phase 3: Advanced Subsystem Initialization")
        print("-" * 80)
        
        success = True
        
        # Container Runtime
        try:
            print("[KERNEL] Initializing container manager...")
            self.container_manager = ContainerManager()
            if self.container_manager.is_available():
                print(f"[KERNEL]   ✓ Container runtime: {self.container_manager.runtime.value}")
            else:
                print("[KERNEL]   ⚠ No container runtime available")
        except Exception as e:
            print(f"[KERNEL]   ⚠ Container manager error: {e}")
        
        # Model Manager
        try:
            print("[KERNEL] Initializing model manager...")
            self.model_manager = ModelManager()
            print("[KERNEL]   ✓ Model manager initialized")
        except Exception as e:
            print(f"[KERNEL]   ⚠ Model manager error: {e}")
        
        # Resource Manager
        try:
            print("[KERNEL] Initializing resource manager...")
            self.resource_manager = ResourceManager()
            self.resource_manager.initialize()
            print("[KERNEL]   ✓ Resource manager initialized")
        except Exception as e:
            print(f"[KERNEL]   ⚠ Resource manager error: {e}")
            success = False
        
        # Job Scheduler
        try:
            print("[KERNEL] Initializing job scheduler...")
            if self.system_specs:
                # Get CPU and GPU counts
                cpu_count = sum(p.cores for p in self.system_specs.processors 
                              if p.processor_type.value == 'cpu')
                gpu_count = sum(1 for p in self.system_specs.processors 
                              if p.processor_type.value == 'gpu')
                gpu_memory = sum(p.memory_gb for p in self.system_specs.processors 
                               if p.processor_type.value == 'gpu' and p.memory_gb)
                
                resource_pool = ResourcePool(
                    total_cpu_cores=float(cpu_count or 4),
                    total_memory_gb=self.system_specs.memory.total_gb,
                    total_gpu_count=gpu_count,
                    total_gpu_memory_gb=gpu_memory or 0
                )
                
                self.scheduler = JobScheduler(resource_pool)
                print(f"[KERNEL]   ✓ Scheduler initialized (CPUs: {cpu_count}, GPUs: {gpu_count})")
            else:
                print("[KERNEL]   ⚠ Cannot initialize scheduler: no system specs")
        except Exception as e:
            print(f"[KERNEL]   ⚠ Scheduler error: {e}")
        
        # Distributed Training Coordinator
        try:
            print("[KERNEL] Initializing distributed coordinator...")
            self.distributed_coordinator = DistributedCoordinator()
            print("[KERNEL]   ✓ Distributed coordinator initialized")
        except Exception as e:
            print(f"[KERNEL]   ⚠ Distributed coordinator error: {e}")
        
        print()
        return success
    
    def _run_diagnostics(self):
        """Run post-boot diagnostics"""
        print("[KERNEL] Phase 4: System Diagnostics")
        print("-" * 80)
        
        if self.monitor:
            snapshot = self.monitor.get_snapshot()
            
            warnings = []
            
            # Resource checks
            if snapshot.cpu_stats.usage_percent > 90:
                warnings.append("⚠ High CPU usage detected")
            
            if snapshot.memory_stats.usage_percent > 90:
                warnings.append("⚠ Low memory available")
            
            for gpu in snapshot.gpu_stats:
                if gpu.memory_percent > 90:
                    warnings.append(f"⚠ GPU {gpu.gpu_id} memory almost full")
                if gpu.temperature_celsius and gpu.temperature_celsius > 85:
                    warnings.append(f"⚠ GPU {gpu.gpu_id} temperature high")
            
            for disk in snapshot.disk_stats:
                if disk.usage_percent > 90:
                    warnings.append(f"⚠ Disk {disk.mount_point} almost full")
            
            if warnings:
                print("[KERNEL] System Warnings:")
                for warning in warnings:
                    print(f"[KERNEL]   {warning}")
            else:
                print("[KERNEL] ✓ All system checks passed")
        
        print()
    
    def _kernel_ready(self):
        """Kernel is ready"""
        print("[KERNEL] Phase 5: System Ready")
        print("-" * 80)
        print("[KERNEL] ✓ AI-OS Advanced Kernel is ready")
        print("[KERNEL] ✓ All systems operational\n")
        
        self._print_capabilities_summary()
    
    def _print_capabilities_summary(self):
        """Print capabilities summary"""
        print("\n" + "╔" + "═"*78 + "╗")
        print("║" + " SYSTEM CAPABILITIES ".center(78, "═") + "║")
        print("╚" + "═"*78 + "╝\n")
        
        print("✅ CORE FEATURES:")
        print("  • Hardware Detection (CPU/GPU/TPU/NPU)")
        print("  • Real-time System Monitoring")
        print("  • Filesystem Management")
        print("  • Network Stack")
        
        print("\n✅ ADVANCED FEATURES:")
        
        if self.container_manager and self.container_manager.is_available():
            print(f"  • Container Runtime ({self.container_manager.runtime.value.upper()})")
        
        if self.model_manager:
            model_count = len(self.model_manager.model_registry)
            print(f"  • AI Model Management ({model_count} models)")
        
        if self.scheduler:
            print(f"  • Job Scheduling & Resource Allocation")
        
        if self.resource_manager:
            print(f"  • GPU Scheduling & Memory Management")
        
        if self.distributed_coordinator:
            print(f"  • Distributed Training Coordination")
        
        print("\n" + "="*80)
        print("\n[KERNEL] Type 'help' for available commands")
        print("[KERNEL] Type 'exit' to shutdown\n")
    
    def run_interactive_shell(self):
        """Run interactive kernel shell"""
        print("[KERNEL] Starting interactive shell...\n")
        
        while self.running:
            try:
                command = input("aios> ").strip()
                
                if not command:
                    continue
                
                # Parse command
                parts = command.split()
                cmd = parts[0].lower()
                args = parts[1:] if len(parts) > 1 else []
                
                # Core commands
                if cmd in ['exit', 'quit']:
                    self._shutdown()
                    break
                
                elif cmd == 'help':
                    self._show_help()
                
                elif cmd == 'status':
                    self._show_status()
                
                elif cmd == 'monitor':
                    self._start_monitoring()
                
                elif cmd == 'hardware':
                    self._show_hardware()
                
                elif cmd == 'network':
                    self._show_network()
                
                elif cmd in ['filesystem', 'fs']:
                    self._show_filesystem()
                
                elif cmd == 'specs':
                    if self.system_specs:
                        detector = HardwareDetector()
                        detector.print_system_report(self.system_specs)
                
                # Advanced commands
                elif cmd == 'containers':
                    self._manage_containers(args)
                
                elif cmd == 'models':
                    self._manage_models(args)
                
                elif cmd == 'jobs':
                    self._manage_jobs(args)
                
                elif cmd == 'resources':
                    self._show_resources()
                
                elif cmd == 'distributed':
                    self._manage_distributed(args)
                
                elif cmd == 'compat':
                    self._compat_check(args)
                else:
                    print(f"Unknown command: {cmd}. Type 'help' for available commands.")
            except KeyboardInterrupt:
                print("\n[KERNEL] Use 'exit' to shutdown")
            except (RuntimeError, ValueError, PermissionError) as err:
                print(f"[KERNEL] Error: {err}")

    def _compat_check(self, args):
        """Check software compatibility"""
        import shlex
        if not args:
            print("Usage: compat [--requirements requirements.txt | --binary path | --container image | --script path] [--os OS] [--arch ARCH]")
            print("Examples:")
            print("  compat --requirements requirements.txt")
            print("  compat --binary ./myprog")
            print("  compat --container ubuntu:22.04")
            print("  compat --script install.sh")
            return
        # Parse args
        import argparse
        parser = argparse.ArgumentParser(prog='compat', add_help=False)
        parser.add_argument('--requirements', type=str)
        parser.add_argument('--binary', type=str)
        parser.add_argument('--container', type=str)
        parser.add_argument('--script', type=str)
        parser.add_argument('--os', type=str)
        parser.add_argument('--arch', type=str)
        try:
            opts = parser.parse_args(args)
        except SystemExit:
            print("Invalid arguments. See 'compat --help'.")
            return
        meta = {}
        if opts.requirements:
            meta['requirements_path'] = opts.requirements
        if opts.binary:
            meta['binary_path'] = opts.binary
        if opts.container:
            meta['container_image'] = opts.container
        if opts.script:
            meta['shell_script_path'] = opts.script
        if opts.os:
            meta['required_os'] = opts.os
        if opts.arch:
            meta['required_arch'] = opts.arch
        if not meta:
            print("No compatibility target specified. Use --requirements, --binary, --container, or --script.")
            return
        report = self.compat_checker.check(meta)
        print("\nCompatibility Report:")
        print("Compatible:" if report.compatible else "NOT Compatible:")
        if report.issues:
            print("Issues:")
            for issue in report.issues:
                print(f"  - {issue}")
        if report.suggestions:
            print("Suggestions:")
            for suggestion in report.suggestions:
                print(f"  - {suggestion}")
        if not report.issues and not report.suggestions:
            print("No issues detected.")

    def _show_help(self):
        """Show help"""
        print("\n" + "="*80)
        print("AVAILABLE COMMANDS")
        print("="*80)
        print("\nCore Commands:")
        print("  help          - Show this help message")
        print("  status        - Show current system status")
        print("  monitor       - Start real-time system monitoring")
        print("  hardware      - Show hardware information")
        print("  network       - Show network configuration")
        print("  filesystem    - Show filesystem information")
        print("  specs         - Show detailed system specifications")
        print("  exit          - Shutdown the system")
        
        print("\nAdvanced Commands:")
        print("  containers    - Manage containers (list, pull, run, stop, rm)")
        print("  models        - Manage AI models (list, load, unload)")
        print("  jobs          - Manage scheduled jobs (list, submit, cancel)")
        print("  resources     - Show resource allocation")
        print("  distributed   - Manage distributed training")
        
        print("\nExamples:")
        print("  aios> containers list")
        print("  aios> models list")
        print("  aios> jobs list")
        print("  aios> monitor")
        print("\n" + "="*80 + "\n")
    
    def _show_status(self):
        """Show system status"""
        if self.monitor:
            snapshot = self.monitor.get_snapshot()
            self.monitor.print_snapshot(snapshot)
    
    def _start_monitoring(self):
        """Start system monitoring"""
        print("[KERNEL] Starting monitoring (press Ctrl+C to stop)...")
        try:
            if self.monitor:
                self.monitor.monitor_continuous(interval_seconds=5, duration_seconds=30)
        except KeyboardInterrupt:
            print("\n[KERNEL] Monitoring stopped")
    
    def _show_hardware(self):
        """Show hardware info"""
        if self.system_specs:
            print("\nHardware Summary:")
            print("-" * 80)
            for proc in self.system_specs.processors:
                print(f"{proc.processor_type.value.upper()}: {proc.vendor.value} {proc.model}")
    
    def _show_network(self):
        """Show network info"""
        if self.net_manager:
            self.net_manager.print_network_summary()
    
    def _show_filesystem(self):
        """Show filesystem info"""
        if self.fs_manager:
            self.fs_manager.print_mount_summary()
    
    def _manage_containers(self, args):
        """Manage containers"""
        if not self.container_manager:
            print("[CONTAINERS] Container manager not available")
            return
        
        if not args:
            print("Usage: containers [list|pull|run|stop|rm|logs]")
            return
        
        subcmd = args[0].lower()
        
        if subcmd == 'list':
            containers = self.container_manager.list_containers(all_containers=True)
            print(f"\nContainers: {len(containers)}")
            for container in containers:
                print(f"  {container.short_id} - {container.name} ({container.status.value})")
        
        elif subcmd == 'images':
            images = self.container_manager.list_images()
            print(f"\nImages: {len(images)}")
            for image in images:
                print(f"  {image.repository}:{image.tag} - {image.size_mb:.1f} MB")
        
        else:
            print(f"Container subcommand '{subcmd}' not yet implemented")
    
    def _manage_models(self, args):
        """Manage models"""
        if not self.model_manager:
            print("[MODELS] Model manager not available")
            return
        
        if not args:
            print("Usage: models [list|load|unload|info]")
            return
        
        subcmd = args[0].lower()
        
        if subcmd == 'list':
            self.model_manager.print_summary()
        
        elif subcmd == 'loaded':
            loaded = self.model_manager.get_loaded_models()
            print(f"\nLoaded Models: {len(loaded)}")
            for model in loaded:
                print(f"  {model.metadata.name} - {model.memory_usage_gb:.2f} GB")
        
        else:
            print(f"Model subcommand '{subcmd}' not yet implemented")
    
    def _manage_jobs(self, args):
        """Manage jobs"""
        if not self.scheduler:
            print("[JOBS] Job scheduler not available")
            return
        
        if not args:
            print("Usage: jobs [list|stats|submit|cancel]")
            return
        
        subcmd = args[0].lower()
        
        if subcmd == 'list':
            self.scheduler.print_summary()
        
        elif subcmd == 'stats':
            stats = self.scheduler.get_queue_stats()
            print(f"\nJob Statistics:")
            print(f"  Queued: {stats['queued']}")
            print(f"  Running: {stats['running']}")
            print(f"  Completed: {stats['completed']}")
            print(f"  Failed: {stats['failed']}")
        
        else:
            print(f"Job subcommand '{subcmd}' not yet implemented")
    
    def _show_resources(self):
        """Show resource allocation"""
        if self.resource_manager:
            self.resource_manager.print_summary()
        else:
            print("[RESOURCES] Resource manager not available")
    
    def _manage_distributed(self, args):
        """Manage distributed training"""
        if not self.distributed_coordinator:
            print("[DISTRIBUTED] Distributed coordinator not available")
            return
        
        if not args:
            print("Usage: distributed [summary|jobs|create]")
            return
        
        subcmd = args[0].lower()
        
        if subcmd == 'summary':
            self.distributed_coordinator.print_summary()
        
        elif subcmd == 'jobs':
            jobs = self.distributed_coordinator.list_jobs()
            print(f"\nDistributed Training Jobs: {len(jobs)}")
            for job in jobs:
                print(f"  {job['name']} - {job['status']}")
        
        else:
            print(f"Distributed subcommand '{subcmd}' not yet implemented")
    
    def _shutdown(self):
        """Shutdown the system"""
        print("\n[KERNEL] Initiating shutdown...")
        print("[KERNEL] Stopping services...")
        print("[KERNEL] Unmounting filesystems...")
        print("[KERNEL] Powering down...")
        print("[KERNEL] ✓ System halted\n")
        self.running = False


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(
        description='AI-OS - Operating System for AI Workloads',
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    
    parser.add_argument(
        '--safe-mode',
        action='store_true',
        help='Boot in safe mode'
    )
    
    parser.add_argument(
        '--verbose', '-v',
        action='store_true',
        help='Verbose output'
    )
    
    parser.add_argument(
        '--no-network',
        action='store_true',
        help='Disable network initialization'
    )
    
    parser.add_argument(
        '--no-fs',
        action='store_true',
        help='Skip filesystem auto-mount'
    )
    
    parser.add_argument(
        '--monitor-only',
        action='store_true',
        help='Run system monitor only (no full boot)'
    )

    parser.add_argument(
        '--advanced',
        action='store_true',
        help='Start advanced kernel (containers/models/distributed)'
    )
    
    args = parser.parse_args()
    
    # Quick monitor mode
    if args.monitor_only:
        print("[KERNEL] Running in monitor-only mode\n")
        monitor = SystemMonitor()
        snapshot = monitor.get_snapshot()
        monitor.print_snapshot(snapshot)
        return 0
    
    # Create boot configuration
    config = BootConfig(
        verbose=args.verbose,
        safe_mode=args.safe_mode,
        auto_mount_filesystems=not args.no_fs,
        start_network=not args.no_network
    )
    
    # Create and start kernel
    kernel = AdvancedAIOSKernel(boot_config=config) if args.advanced else AIOSKernel(boot_config=config)
    
    if kernel.start():
        # Run interactive shell
        kernel.run_interactive_shell()
        return 0
    else:
        print("[KERNEL] Failed to start kernel")
        return 1


if __name__ == "__main__":
    sys.exit(main())
