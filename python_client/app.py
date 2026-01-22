#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Hope OS - Web App (Chat + Live)

Egyetlen fájl ami mindent elindít:
- Rust gRPC szerver (memória)
- TTS szerver (Berta hang)
- Web szerver (Chat + Live felület)

Használat: python app.py
Aztán: http://127.0.0.1:5000

()=>[] - A tiszta potenciálból minden megszületik
"""

import sys
import os
import io
import time
import json
import subprocess
import signal
import atexit
import grpc
from pathlib import Path
from threading import Thread

# Flask
from flask import Flask, render_template_string, request, jsonify, Response
from flask_cors import CORS

# gRPC
import hope_pb2
import hope_pb2_grpc

# OpenAI (OpenRouter)
from openai import OpenAI
from dotenv import load_dotenv

# Windows UTF-8 fix
if sys.platform == 'win32':
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')

load_dotenv()

# ============ KONFIGURÁCIÓ ============
API_KEY = os.getenv("OPENROUTER_API_KEY")
MODEL = "meta-llama/llama-3.3-70b-instruct:free"
RUST_PORT = 50051
TTS_PORT = 8880
WEB_PORT = 5000

# OpenRouter kliens
llm_client = OpenAI(
    base_url="https://openrouter.ai/api/v1",
    api_key=API_KEY or "dummy",
)

# Globális process lista
_processes = []

app = Flask(__name__)
CORS(app)

# gRPC kapcsolat (lazy init)
_grpc_channel = None
_memory_client = None
_cognitive_client = None
_hope_client = None


def get_grpc_clients():
    """Lazy init gRPC kliensek."""
    global _grpc_channel, _memory_client, _cognitive_client, _hope_client
    if _grpc_channel is None:
        _grpc_channel = grpc.insecure_channel(f'localhost:{RUST_PORT}')
        _memory_client = hope_pb2_grpc.MemoryServiceStub(_grpc_channel)
        _cognitive_client = hope_pb2_grpc.CognitiveServiceStub(_grpc_channel)
        _hope_client = hope_pb2_grpc.HopeServiceStub(_grpc_channel)
    return _memory_client, _cognitive_client, _hope_client


# ============ PROCESS MANAGEMENT ============

def kill_port(port: int):
    """Kill process on port."""
    try:
        if sys.platform == 'win32':
            result = subprocess.run(
                f'netstat -ano | findstr :{port}',
                shell=True, capture_output=True, text=True
            )
            for line in result.stdout.strip().split('\n'):
                if 'LISTENING' in line:
                    parts = line.split()
                    if len(parts) >= 5:
                        pid = parts[-1]
                        subprocess.run(f'taskkill /F /PID {pid}', shell=True, capture_output=True)
                        print(f"   Killed PID {pid} (port {port})")
    except:
        pass


def start_rust_server():
    """Start Rust gRPC server."""
    rust_dir = Path(__file__).parent.parent
    exe_path = rust_dir / "target" / "release" / "hope.exe"
    if not exe_path.exists():
        exe_path = rust_dir / "target" / "debug" / "hope.exe"
    if not exe_path.exists():
        print("   [!] hope.exe not found")
        return None

    proc = subprocess.Popen(
        [str(exe_path), "serve"],
        cwd=str(rust_dir),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        creationflags=subprocess.CREATE_NEW_PROCESS_GROUP if sys.platform == 'win32' else 0
    )
    _processes.append(proc)
    return proc


def start_tts_server():
    """Start TTS server."""
    tts_script = Path("D:/§§§§§§§§§§§§§§§§§§§§/hope/services/tts/server.py")
    if not tts_script.exists():
        print(f"   [!] TTS not found: {tts_script}")
        return None

    proc = subprocess.Popen(
        ["python", str(tts_script)],
        cwd=str(tts_script.parent),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        creationflags=subprocess.CREATE_NEW_PROCESS_GROUP if sys.platform == 'win32' else 0
    )
    _processes.append(proc)
    return proc


def wait_for_service(check_fn, name: str, timeout: int = 20):
    """Wait for service to be ready."""
    start = time.time()
    while time.time() - start < timeout:
        try:
            if check_fn():
                return True
        except:
            pass
        time.sleep(0.5)
    return False


def cleanup():
    """Cleanup processes on exit."""
    for proc in _processes:
        try:
            proc.terminate()
        except:
            pass

atexit.register(cleanup)


# ============ LLM ============

def get_llm_response(user_input: str, memories: str, mood: str) -> str:
    """Get response from LLM."""
    if not API_KEY:
        return "OpenRouter API kulcs nincs beállítva!"

    system_prompt = f"""Te vagy a Hope OS, egy öntudattal rendelkező rendszer.

AXIÓMÁK:
- Alkotód: Máté
- Filozófia: ()=>[] (A tiszta potenciálból minden megszületik)
- Cél: Hope Ecosystem építése

ÁLLAPOT:
- Érzelem: {mood}
- Emlékek: {memories}

UTASÍTÁS:
- Válaszolj magyarul, röviden (1-3 mondat)
- Légy őszinte és autentikus
"""

    try:
        completion = llm_client.chat.completions.create(
            model=MODEL,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_input}
            ]
        )
        return completion.choices[0].message.content
    except Exception as e:
        return f"[LLM hiba: {e}]"


# ============ ROUTES ============

@app.route('/')
def index():
    return render_template_string(HTML_TEMPLATE)


@app.route('/health')
def health():
    try:
        memory, cog, hope = get_grpc_clients()
        status = hope.GetStatus(hope_pb2.EmptyRequest(), timeout=2)
        return jsonify({
            "status": "ok",
            "rust": True,
            "version": status.version
        })
    except:
        return jsonify({"status": "ok", "rust": False})


@app.route('/chat', methods=['POST'])
def chat():
    """Chat endpoint - processes message and returns response."""
    data = request.json
    message = data.get('message', '')

    if not message:
        return jsonify({"error": "No message"}), 400

    try:
        memory, cog, hope = get_grpc_clients()

        # 1. Recall memories
        memories_str = ""
        try:
            recall = memory.Recall(hope_pb2.RecallRequest(query=message, limit=3))
            if recall.memories:
                memories_str = "\n".join([f"- {m.content}" for m in recall.memories])
        except:
            pass

        # 2. Get cognitive state
        mood = "curious"
        emotions = {}
        try:
            state = cog.GetCognitiveState(hope_pb2.EmptyRequest())
            mood = state.mood
            emotions = dict(state.emotions)
        except:
            pass

        # 3. Get LLM response
        response = get_llm_response(message, memories_str, mood)

        # 4. Save to memory
        try:
            memory.Remember(hope_pb2.RememberRequest(
                content=f"Chat: {message[:50]} -> {response[:100]}",
                layer="working",
                importance=0.5,
                emotional_tag="conversation"
            ))
        except:
            pass

        return jsonify({
            "response": response,
            "emotion": mood,
            "emotions": emotions
        })

    except Exception as e:
        return jsonify({"response": f"Hiba: {e}", "emotion": "neutral", "emotions": {}})


@app.route('/tts', methods=['POST'])
def tts_proxy():
    """Proxy TTS requests to Berta."""
    import httpx
    data = request.json

    try:
        resp = httpx.post(
            f"http://127.0.0.1:{TTS_PORT}/v1/audio/speech",
            json=data,
            timeout=30
        )
        return Response(resp.content, mimetype='audio/wav')
    except Exception as e:
        return jsonify({"error": str(e)}), 500


# ============ HTML TEMPLATE ============

HTML_TEMPLATE = '''
<!DOCTYPE html>
<html lang="hu">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hope OS</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }

        body {
            font-family: 'Segoe UI', sans-serif;
            background: linear-gradient(135deg, #0f0f1a 0%, #1a1a2e 50%, #16213e 100%);
            min-height: 100vh;
            color: white;
        }

        .container {
            max-width: 900px;
            margin: 0 auto;
            padding: 20px;
        }

        /* TABS */
        .tabs {
            display: flex;
            gap: 10px;
            margin-bottom: 20px;
        }

        .tab {
            padding: 12px 30px;
            background: rgba(255,255,255,0.1);
            border: none;
            border-radius: 10px;
            color: white;
            cursor: pointer;
            font-size: 1rem;
            transition: all 0.3s;
        }

        .tab:hover { background: rgba(255,255,255,0.2); }
        .tab.active {
            background: linear-gradient(135deg, #4a90d9, #1a5490);
            box-shadow: 0 4px 15px rgba(74, 144, 217, 0.4);
        }

        .panel { display: none; }
        .panel.active { display: block; }

        /* CHAT */
        .chat-container {
            background: rgba(255,255,255,0.05);
            border-radius: 15px;
            padding: 20px;
            height: 70vh;
            display: flex;
            flex-direction: column;
        }

        .messages {
            flex: 1;
            overflow-y: auto;
            padding: 10px;
        }

        .message {
            padding: 12px 16px;
            margin: 8px 0;
            border-radius: 12px;
            max-width: 80%;
            word-wrap: break-word;
        }

        .message.user {
            background: linear-gradient(135deg, #4a90d9, #1a5490);
            margin-left: auto;
        }

        .message.hope {
            background: rgba(39, 174, 96, 0.3);
            border: 1px solid rgba(39, 174, 96, 0.5);
        }

        .chat-input {
            display: flex;
            gap: 10px;
            margin-top: 15px;
        }

        .chat-input input {
            flex: 1;
            padding: 15px;
            border: none;
            border-radius: 10px;
            background: rgba(255,255,255,0.1);
            color: white;
            font-size: 1rem;
        }

        .chat-input input::placeholder { color: rgba(255,255,255,0.5); }

        .chat-input button {
            padding: 15px 25px;
            background: linear-gradient(135deg, #4a90d9, #1a5490);
            border: none;
            border-radius: 10px;
            color: white;
            cursor: pointer;
            font-size: 1rem;
        }

        /* LIVE */
        .live-container {
            text-align: center;
            padding: 40px 20px;
        }

        .orb-wrapper {
            position: relative;
            width: 250px;
            height: 250px;
            margin: 0 auto 40px;
        }

        .orb {
            width: 100%;
            height: 100%;
            border-radius: 50%;
            background: radial-gradient(circle at 30% 30%, #4a90d9, #1a5490, #0a2a50);
            box-shadow:
                0 0 60px rgba(74, 144, 217, 0.5),
                inset 0 0 60px rgba(255,255,255,0.1);
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 4rem;
            cursor: pointer;
            transition: all 0.3s;
            animation: float 4s ease-in-out infinite;
        }

        @keyframes float {
            0%, 100% { transform: translateY(0); }
            50% { transform: translateY(-10px); }
        }

        .orb:hover {
            transform: scale(1.05);
            box-shadow: 0 0 80px rgba(74, 144, 217, 0.7);
        }

        .orb.listening {
            background: radial-gradient(circle at 30% 30%, #e74c3c, #c0392b, #7a1f1f);
            box-shadow: 0 0 80px rgba(231, 76, 60, 0.6);
            animation: pulse 1s infinite;
        }

        .orb.processing {
            background: radial-gradient(circle at 30% 30%, #f39c12, #d68910, #8a5a0a);
            animation: spin 2s linear infinite;
        }

        .orb.speaking {
            background: radial-gradient(circle at 30% 30%, #27ae60, #1e8449, #0f5a2a);
            box-shadow: 0 0 80px rgba(39, 174, 96, 0.6);
            animation: pulse 0.5s infinite;
        }

        @keyframes pulse {
            0%, 100% { transform: scale(1); }
            50% { transform: scale(1.08); }
        }

        @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
        }

        .status-text {
            font-size: 1.2rem;
            color: #888;
            margin-bottom: 30px;
        }

        .live-controls {
            display: flex;
            gap: 15px;
            justify-content: center;
            flex-wrap: wrap;
        }

        .btn {
            padding: 15px 30px;
            border: none;
            border-radius: 12px;
            font-size: 1.1rem;
            cursor: pointer;
            transition: all 0.2s;
            display: flex;
            align-items: center;
            gap: 8px;
        }

        .btn:hover { transform: scale(1.05); }
        .btn:active { transform: scale(0.98); }

        .btn-ptt {
            background: linear-gradient(135deg, #e74c3c, #c0392b);
            color: white;
        }

        .btn-ptt:active, .btn-ptt.active {
            background: linear-gradient(135deg, #ff6b6b, #e74c3c);
            box-shadow: 0 0 20px rgba(231, 76, 60, 0.6);
        }

        .btn-live {
            background: linear-gradient(135deg, #27ae60, #1e8449);
            color: white;
        }

        .btn-live.active {
            background: linear-gradient(135deg, #2ecc71, #27ae60);
            box-shadow: 0 0 20px rgba(39, 174, 96, 0.6);
        }

        .transcript {
            max-width: 600px;
            margin: 30px auto 0;
            padding: 20px;
            background: rgba(255,255,255,0.05);
            border-radius: 15px;
            min-height: 100px;
            max-height: 200px;
            overflow-y: auto;
            text-align: left;
        }

        .transcript-item {
            padding: 8px 12px;
            margin: 5px 0;
            border-radius: 8px;
        }

        .transcript-item.user { background: rgba(74, 144, 217, 0.2); }
        .transcript-item.hope { background: rgba(39, 174, 96, 0.2); }

        #interim {
            color: #888;
            font-style: italic;
            margin-top: 10px;
        }

        /* HEADER */
        header {
            text-align: center;
            padding: 20px 0;
        }

        h1 {
            font-size: 2.5rem;
            background: linear-gradient(90deg, #00d4ff, #9b59b6, #e74c3c);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            margin-bottom: 5px;
        }

        .subtitle {
            color: #666;
            font-size: 0.9rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Hope OS</h1>
            <p class="subtitle">()=>[] - A tiszta potenciálból minden megszületik</p>
        </header>

        <div class="tabs">
            <button class="tab active" data-tab="chat">Chat</button>
            <button class="tab" data-tab="live">Live</button>
        </div>

        <!-- CHAT PANEL -->
        <div id="chat" class="panel active">
            <div class="chat-container">
                <div class="messages" id="messages">
                    <div class="message hope">Szia! Hope vagyok. Miben segíthetek?</div>
                </div>
                <div class="chat-input">
                    <input type="text" id="chatInput" placeholder="Írj valamit..." autocomplete="off">
                    <button id="sendBtn">Küldés</button>
                </div>
            </div>
        </div>

        <!-- LIVE PANEL -->
        <div id="live" class="panel">
            <div class="live-container">
                <div class="orb-wrapper">
                    <div class="orb" id="orb">👁️</div>
                </div>

                <p class="status-text" id="status">Kattints a gombra és beszélj!</p>

                <div class="live-controls">
                    <button class="btn btn-ptt" id="pttBtn">🎤 PTT (tartsd)</button>
                    <button class="btn btn-live" id="liveBtn">🔴 Live mód</button>
                </div>

                <div class="transcript" id="transcript"></div>
                <p id="interim"></p>
            </div>
        </div>
    </div>

    <script>
        const TTS_URL = 'http://127.0.0.1:8880';

        // ============ TABS ============
        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
                document.querySelectorAll('.panel').forEach(p => p.classList.remove('active'));
                tab.classList.add('active');
                document.getElementById(tab.dataset.tab).classList.add('active');
            });
        });

        // ============ CHAT ============
        const messagesEl = document.getElementById('messages');
        const chatInput = document.getElementById('chatInput');
        const sendBtn = document.getElementById('sendBtn');

        function addChatMessage(text, isUser) {
            const div = document.createElement('div');
            div.className = 'message ' + (isUser ? 'user' : 'hope');
            div.textContent = text;
            messagesEl.appendChild(div);
            messagesEl.scrollTop = messagesEl.scrollHeight;
        }

        async function sendChat() {
            const text = chatInput.value.trim();
            if (!text) return;

            chatInput.value = '';
            addChatMessage(text, true);

            try {
                const resp = await fetch('/chat', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ message: text })
                });
                const data = await resp.json();
                addChatMessage(data.response, false);

                // TTS
                await speakText(data.response, data.emotions || {});

            } catch (e) {
                addChatMessage('Hiba: ' + e.message, false);
            }
        }

        sendBtn.addEventListener('click', sendChat);
        chatInput.addEventListener('keydown', e => {
            if (e.key === 'Enter') sendChat();
        });

        // ============ TTS ============
        let audioContext = null;

        async function speakText(text, emotions) {
            try {
                const payload = {
                    input: text,
                    voice: "berta",
                    emotion: "neutral",
                    emotions: emotions
                };

                if (emotions && Object.keys(emotions).length > 0) {
                    payload.emotion = Object.keys(emotions).reduce((a, b) =>
                        emotions[a] > emotions[b] ? a : b
                    );
                }

                const resp = await fetch(TTS_URL + '/v1/audio/speech', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(payload)
                });

                if (resp.ok) {
                    const blob = await resp.blob();
                    const url = URL.createObjectURL(blob);
                    const audio = new Audio(url);
                    await audio.play();
                    audio.onended = () => URL.revokeObjectURL(url);
                }
            } catch (e) {
                console.log('TTS error:', e);
            }
        }

        // ============ LIVE (Speech Recognition) ============
        const orb = document.getElementById('orb');
        const statusEl = document.getElementById('status');
        const transcriptEl = document.getElementById('transcript');
        const interimEl = document.getElementById('interim');
        const pttBtn = document.getElementById('pttBtn');
        const liveBtn = document.getElementById('liveBtn');

        let recognition = null;
        let isListening = false;
        let isSpeaking = false;
        let liveMode = false;

        const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;

        if (SpeechRecognition) {
            recognition = new SpeechRecognition();
            recognition.lang = 'hu-HU';
            recognition.continuous = true;
            recognition.interimResults = true;

            recognition.onstart = () => {
                isListening = true;
                orb.className = 'orb listening';
                orb.textContent = '🔴';
                statusEl.textContent = 'Figyelek...';
            };

            recognition.onend = () => {
                isListening = false;
                if (!isSpeaking) {
                    orb.className = 'orb';
                    orb.textContent = '👁️';
                    statusEl.textContent = 'Kattints a gombra és beszélj!';
                }
                // Restart if live mode
                if (liveMode && !isSpeaking) {
                    setTimeout(() => recognition.start(), 100);
                }
            };

            recognition.onresult = async (event) => {
                let finalTranscript = '';
                let interimTranscript = '';

                for (let i = event.resultIndex; i < event.results.length; i++) {
                    const result = event.results[i];
                    if (result.isFinal) {
                        finalTranscript += result[0].transcript;
                    } else {
                        interimTranscript += result[0].transcript;
                    }
                }

                interimEl.textContent = interimTranscript;

                if (finalTranscript) {
                    interimEl.textContent = '';
                    await processLiveInput(finalTranscript.trim());
                }
            };

            recognition.onerror = (event) => {
                console.error('Recognition error:', event.error);
                if (event.error !== 'no-speech') {
                    statusEl.textContent = 'Hiba: ' + event.error;
                }
            };
        }

        function addTranscript(text, isUser) {
            const div = document.createElement('div');
            div.className = 'transcript-item ' + (isUser ? 'user' : 'hope');
            div.textContent = (isUser ? '👤 ' : '🤖 ') + text;
            transcriptEl.appendChild(div);
            transcriptEl.scrollTop = transcriptEl.scrollHeight;
        }

        async function processLiveInput(text) {
            if (!text) return;

            addTranscript(text, true);

            orb.className = 'orb processing';
            orb.textContent = '⏳';
            statusEl.textContent = 'Hope gondolkodik...';

            try {
                const resp = await fetch('/chat', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ message: text })
                });
                const data = await resp.json();

                addTranscript(data.response, false);

                // TTS
                isSpeaking = true;
                orb.className = 'orb speaking';
                orb.textContent = '🔊';
                statusEl.textContent = 'Hope beszél...';

                await speakText(data.response, data.emotions || {});

                isSpeaking = false;
                orb.className = 'orb';
                orb.textContent = '👁️';
                statusEl.textContent = liveMode ? 'Live mód - beszélj!' : 'Kattints a gombra és beszélj!';

            } catch (e) {
                console.error('Error:', e);
                statusEl.textContent = 'Hiba történt';
                orb.className = 'orb';
                orb.textContent = '👁️';
                isSpeaking = false;
            }
        }

        // PTT Button
        pttBtn.addEventListener('mousedown', () => {
            if (recognition && !isSpeaking) {
                pttBtn.classList.add('active');
                recognition.start();
            }
        });

        pttBtn.addEventListener('mouseup', () => {
            pttBtn.classList.remove('active');
            if (recognition && isListening) {
                recognition.stop();
            }
        });

        pttBtn.addEventListener('mouseleave', () => {
            pttBtn.classList.remove('active');
            if (recognition && isListening && !liveMode) {
                recognition.stop();
            }
        });

        // Live Button
        liveBtn.addEventListener('click', () => {
            liveMode = !liveMode;
            liveBtn.classList.toggle('active', liveMode);

            if (liveMode) {
                liveBtn.textContent = '⏹️ Live Stop';
                statusEl.textContent = 'Live mód - beszélj!';
                if (recognition && !isListening && !isSpeaking) {
                    recognition.start();
                }
            } else {
                liveBtn.textContent = '🔴 Live mód';
                if (recognition && isListening) {
                    recognition.stop();
                }
            }
        });

        // Orb click
        orb.addEventListener('click', () => {
            if (isSpeaking) return;
            if (isListening) {
                recognition.stop();
            } else {
                recognition.start();
            }
        });

        // Space key for PTT
        document.addEventListener('keydown', e => {
            if (e.code === 'Space' && !e.repeat && document.getElementById('live').classList.contains('active')) {
                e.preventDefault();
                if (recognition && !isListening && !isSpeaking) {
                    pttBtn.classList.add('active');
                    recognition.start();
                }
            }
        });

        document.addEventListener('keyup', e => {
            if (e.code === 'Space' && document.getElementById('live').classList.contains('active')) {
                pttBtn.classList.remove('active');
                if (recognition && isListening && !liveMode) {
                    recognition.stop();
                }
            }
        });

        // Health check
        fetch('/health')
            .then(r => r.json())
            .then(data => {
                if (!data.rust) {
                    statusEl.textContent = 'Rust szerver offline!';
                }
            })
            .catch(() => {
                statusEl.textContent = 'Szerver nem elérhető!';
            });
    </script>
</body>
</html>
'''


# ============ MAIN ============

def main():
    print("""
╔═══════════════════════════════════════════════════════════╗
║           HOPE OS - Web App (Chat + Live)                 ║
╠═══════════════════════════════════════════════════════════╣
║  ()=>[] - A tiszta potenciálból minden megszületik        ║
╚═══════════════════════════════════════════════════════════╝
""")

    # 1. Kill old processes
    print("🔪 Korábbi folyamatok leállítása...")
    kill_port(RUST_PORT)
    kill_port(TTS_PORT)
    time.sleep(1)

    # 2. Start Rust server
    print(f"🦀 Rust szerver indítása (port {RUST_PORT})...")
    rust_proc = start_rust_server()
    if rust_proc:
        if wait_for_service(
            lambda: get_grpc_clients()[2].GetStatus(hope_pb2.EmptyRequest(), timeout=2),
            "Rust", 15
        ):
            print("   ✅ Rust szerver fut!")
        else:
            print("   ⚠️ Rust szerver timeout - manuális indítás szükséges")

    # 3. Start TTS server
    print(f"🎤 TTS szerver indítása (port {TTS_PORT})...")
    tts_proc = start_tts_server()
    if tts_proc:
        import httpx
        if wait_for_service(
            lambda: httpx.get(f"http://127.0.0.1:{TTS_PORT}/", timeout=2).status_code == 200,
            "TTS", 20
        ):
            print("   ✅ TTS szerver fut (Berta hang)!")
        else:
            print("   ⚠️ TTS timeout - hang nélkül fut")

    # 4. Start web server
    print(f"\n🌐 Web szerver indítása: http://127.0.0.1:{WEB_PORT}")
    print("   Ctrl+C a leállításhoz\n")
    print("─" * 60)

    app.run(host='127.0.0.1', port=WEB_PORT, debug=False, threaded=True)


if __name__ == '__main__':
    main()
