#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Hope OS - Smart Search Teszt

Kulonbozo keresesek tesztelese a Hope memoriajabol.
"""

import sys
import grpc
import hope_pb2
import hope_pb2_grpc

# Windows UTF-8 fix
if sys.platform == 'win32':
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')


def test_recall():
    print("""
╔═══════════════════════════════════════════════════════════╗
║         HOPE OS - Smart Search Teszt                      ║
╚═══════════════════════════════════════════════════════════╝
""")

    channel = grpc.insecure_channel('localhost:50051')
    memory_client = hope_pb2_grpc.MemoryServiceStub(channel)

    # Tesztelendo kerdesek
    queries = [
        "ki vagy te",
        "alkotod",
        "Mate",
        "cel",
        "Rust",
        "erzelem",
        "Claude",
        "filozofia",
    ]

    print("🔍 Keresesek tesztelese...\n")

    for query in queries:
        print(f"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print(f"Q: \"{query}\"")
        print(f"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")

        try:
            request = hope_pb2.RecallRequest(
                query=query,
                layer="",  # Minden layer
                limit=3
            )
            response = memory_client.Recall(request)

            if response.total > 0:
                print(f"   Talalatok: {response.total}")
                for mem in response.memories:
                    short = mem.content[:60] + "..." if len(mem.content) > 60 else mem.content
                    print(f"   ✅ {short}")
                    print(f"      [layer: {mem.layer}, importance: {mem.importance:.0%}]")
            else:
                print(f"   ❌ Nincs talalat")

        except grpc.RpcError as e:
            print(f"   ❌ Hiba: {e.details()}")

        print()

    print("═" * 60)
    print("Smart Search teszt befejezve!")
    print("═" * 60)


if __name__ == "__main__":
    test_recall()
