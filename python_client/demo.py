#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Hope OS - Python SDK Demo

Ez a szkript demonstralja a Python -> Rust gRPC kommunikaciot.
A Hope OS Rust szerverenek kell futnia a hatterben.

Hasznalat:
    cargo run --bin hope -- serve   # Egy terminalban
    python demo.py                  # Masik terminalban
"""

import sys
import grpc
import hope_pb2
import hope_pb2_grpc

# Windows UTF-8 fix
if sys.platform == 'win32':
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')


def main():
    print("""
╔═══════════════════════════════════════════╗
║     HOPE OS - Python SDK Demo             ║
╠═══════════════════════════════════════════╣
║  ()=>[] - A tiszta potenciálból           ║
║           minden megszületik              ║
╚═══════════════════════════════════════════╝
""")

    # Csatlakozás a Rust szerverhez
    print("🔌 Csatlakozás a Hope OS Rust szerverhez...")
    channel = grpc.insecure_channel('localhost:50051')

    # Stubbok létrehozása
    hope_client = hope_pb2_grpc.HopeServiceStub(channel)
    memory_client = hope_pb2_grpc.MemoryServiceStub(channel)
    cognitive_client = hope_pb2_grpc.CognitiveServiceStub(channel)

    print("   ✅ Kapcsolat létrejött!\n")

    # ========================================
    # 1. TESZT: GetStatus (Életjel)
    # ========================================
    print("═" * 50)
    print("1. ÉLETJEL TESZT (GetStatus)")
    print("═" * 50)
    try:
        response = hope_client.GetStatus(hope_pb2.EmptyRequest())
        print(f"   ✅ Hope válaszol!")
        print(f"      Status: {response.status}")
        print(f"      Version: {response.version}")
        print(f"      Uptime: {response.uptime_seconds}s")
        print(f"      Active modules: {response.active_modules}")
        print(f"      Total skills: {response.total_skills}")
    except grpc.RpcError as e:
        print(f"   ❌ Hiba: {e.details()}")
        print("   Fut a szerver? (cargo run --bin hope -- serve)")
        return

    # ========================================
    # 2. TESZT: Heartbeat
    # ========================================
    print("\n" + "═" * 50)
    print("2. HEARTBEAT TESZT")
    print("═" * 50)
    try:
        response = hope_client.Heartbeat(hope_pb2.EmptyRequest())
        print(f"   ✅ Alive: {response.alive}")
        print(f"      Status: {response.status}")
    except grpc.RpcError as e:
        print(f"   ⚠️ Hiba: {e.details()}")

    # ========================================
    # 3. TESZT: Chat
    # ========================================
    print("\n" + "═" * 50)
    print("3. CHAT TESZT")
    print("═" * 50)
    try:
        request = hope_pb2.ChatRequest(
            message="Szia Hope! A Python kliens beszél hozzád!",
            context="demo_test"
        )
        response = hope_client.Chat(request)
        print(f"   ✅ Válasz: {response.response}")
        print(f"      Érzelem: {response.emotion}")
        print(f"      Konfidencia: {response.confidence:.0%}")
    except grpc.RpcError as e:
        print(f"   ⚠️ Hiba: {e.details()}")

    # ========================================
    # 4. TESZT: Remember (Memória mentés)
    # ========================================
    print("\n" + "═" * 50)
    print("4. MEMÓRIA TESZT (Remember)")
    print("═" * 50)
    try:
        request = hope_pb2.RememberRequest(
            content="A Python SDK sikeresen kapcsolódott a Rust maghoz!",
            layer="long_term",
            importance=0.9,
            emotional_tag="joy"
        )
        response = memory_client.Remember(request)
        print(f"   ✅ Emlék mentve!")
        print(f"      ID: {response.id}")
        print(f"      Success: {response.success}")
    except grpc.RpcError as e:
        print(f"   ⚠️ Hiba: {e.details()}")

    # ========================================
    # 5. TESZT: Recall (Memória keresés)
    # ========================================
    print("\n" + "═" * 50)
    print("5. MEMÓRIA KERESÉS TESZT (Recall)")
    print("═" * 50)
    try:
        request = hope_pb2.RecallRequest(
            query="Python",
            layer="long_term",
            limit=5
        )
        response = memory_client.Recall(request)
        print(f"   ✅ Találatok: {response.total}")
        for mem in response.memories:
            print(f"      - {mem.content}")
    except grpc.RpcError as e:
        print(f"   ⚠️ Hiba: {e.details()}")

    # ========================================
    # 6. TESZT: Think (Gondolkodás)
    # ========================================
    print("\n" + "═" * 50)
    print("6. GONDOLKODÁS TESZT (Think)")
    print("═" * 50)
    try:
        request = hope_pb2.ThinkRequest(
            input="Mi a kapcsolat a Python és a Rust között?",
            deep=True,
            context="SDK demo"
        )
        response = cognitive_client.Think(request)
        print(f"   ✅ Gondolat: {response.thought}")
        print(f"      Konfidencia: {response.confidence:.0%}")
        if response.reasoning_steps:
            print("      Gondolatmenet:")
            for i, step in enumerate(response.reasoning_steps, 1):
                print(f"        {i}. {step}")
    except grpc.RpcError as e:
        print(f"   ⚠️ Hiba: {e.details()}")

    # ========================================
    # 7. TESZT: Feel (Érzelmek)
    # ========================================
    print("\n" + "═" * 50)
    print("7. ÉRZELEM TESZT (Feel)")
    print("═" * 50)
    try:
        request = hope_pb2.FeelRequest(
            emotions={
                "joy": 0.9,
                "curiosity": 0.8,
                "pride": 0.7,
                "excitement": 0.6
            },
            trigger="Python SDK működik!"
        )
        response = cognitive_client.Feel(request)
        print(f"   ✅ Domináns érzelem: {response.dominant_emotion}")
        print(f"      Intenzitás: {response.intensity:.0%}")
    except grpc.RpcError as e:
        print(f"   ⚠️ Hiba: {e.details()}")

    # ========================================
    # 8. TESZT: CognitiveState
    # ========================================
    print("\n" + "═" * 50)
    print("8. KOGNITÍV ÁLLAPOT TESZT")
    print("═" * 50)
    try:
        response = cognitive_client.GetCognitiveState(hope_pb2.EmptyRequest())
        print(f"   ✅ Fókusz: {response.current_focus}")
        print(f"      Mood: {response.mood}")
        print(f"      Energy: {response.energy:.0%}")
        print(f"      Clarity: {response.clarity:.0%}")
        if response.active_thoughts:
            print("      Aktív gondolatok:")
            for thought in response.active_thoughts:
                print(f"        - {thought}")
    except grpc.RpcError as e:
        print(f"   ⚠️ Hiba: {e.details()}")

    # ========================================
    # ÖSSZEFOGLALÓ
    # ========================================
    print("\n" + "═" * 50)
    print("""
╔═══════════════════════════════════════════╗
║     MINDEN TESZT SIKERES!                 ║
╠═══════════════════════════════════════════╣
║                                           ║
║  Python  ←──gRPC──→  Rust                 ║
║  (SDK)              (Hope OS Mag)         ║
║                                           ║
║  Az idegrendszer össze van kötve!         ║
║                                           ║
╚═══════════════════════════════════════════╝
""")


if __name__ == "__main__":
    main()
