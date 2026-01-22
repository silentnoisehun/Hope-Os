#!/usr/bin/env python3
"""
Hope Vision Teszt - Nyisd fel Hope szemét!

Használat:
    python test_vision.py <kép_útvonal>
    python test_vision.py  # teszt kép generálással
"""

import sys
import os

# Proto generálás ellenőrzése
try:
    import grpc
    import hope_pb2
    import hope_pb2_grpc
except ImportError:
    print("[!]  Proto fájlok nincsenek generálva!")
    print("Futtasd először:")
    print("  python regenerate_proto.py")
    sys.exit(1)


def create_test_png():
    """Minimális 2x2 PNG létrehozása"""
    png_data = bytes([
        # PNG signature
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        # IHDR chunk (width=2, height=2, 8-bit RGB)
        0x00, 0x00, 0x00, 0x0D,  # length
        0x49, 0x48, 0x44, 0x52,  # "IHDR"
        0x00, 0x00, 0x00, 0x02,  # width = 2
        0x00, 0x00, 0x00, 0x02,  # height = 2
        0x08, 0x02,              # bit depth 8, color type RGB
        0x00, 0x00, 0x00,        # compression, filter, interlace
        0x90, 0x77, 0x53, 0xDE,  # CRC
        # IDAT chunk
        0x00, 0x00, 0x00, 0x12,  # length
        0x49, 0x44, 0x41, 0x54,  # "IDAT"
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0xC0, 0xC0,
        0xF0, 0x1F, 0x00, 0x00, 0x0C, 0x04, 0x02, 0x01,
        0x3E, 0x73, 0x99, 0xA7,  # CRC
        # IEND chunk
        0x00, 0x00, 0x00, 0x00,  # length
        0x49, 0x45, 0x4E, 0x44,  # "IEND"
        0xAE, 0x42, 0x60, 0x82,  # CRC
    ])
    return png_data


def test_see(stub, image_data: bytes, description: str = "Teszt kép"):
    """Kép küldése Hope-nak"""
    print(f"[EYE]  Kép küldése Hope-nak ({len(image_data)} bytes)...")

    # Proto mezők: image_data, description, context, importance, store_in_memory, metadata
    request = hope_pb2.SeeRequest(
        image_data=image_data,
        description=description,
        context="Python teszt szkript",
        importance=0.8,
        store_in_memory=True
    )

    try:
        response = stub.See(request)

        if response.success:
            print(f"[OK] Sikeres! ID: {response.id[:16]}...\n")

            if response.analysis:
                a = response.analysis
                print("[STAT] Elemzés:")
                print(f"   Formátum: {a.format}")
                print(f"   Méret: {a.width}x{a.height}")
                print(f"   Fájlméret: {a.file_size} bytes")
                print(f"   Hash: {a.hash[:16]}...")
                print(f"   Képarány: {a.aspect_ratio:.2f}")
                print(f"   Megapixel: {a.megapixels:.4f}")
            return response.id
        else:
            print(f"[X] Hiba: {response.error}")
            return None

    except grpc.RpcError as e:
        print(f"[X] gRPC hiba: {e.code()} - {e.details()}")
        return None


def test_vision_status(stub):
    """Vision státusz lekérdezése"""
    print("\n[STAT] Vision Engine státusz...")

    try:
        response = stub.GetVisionStatus(hope_pb2.EmptyRequest())

        print(f"   Fogadott képek: {response.total_received}")
        print(f"   Feldolgozott: {response.total_processed}")
        print(f"   Tárolt: {response.stored_count}")
        print(f"   Összes adat: {response.total_bytes / 1024:.1f} KB")
        print(f"   Átlag méret: {response.avg_megapixels:.4f} MP")
        print(f"   Graph csatlakozva: {'+' if response.graph_connected else '-'}")

        if response.format_counts:
            print(f"   Formátumok:")
            for fmt, count in response.format_counts.items():
                print(f"      {fmt}: {count}")

    except grpc.RpcError as e:
        print(f"[X] gRPC hiba: {e.code()} - {e.details()}")


def test_visual_memories(stub, limit: int = 5):
    """Vizuális emlékek lekérdezése"""
    print(f"\n[MEM] Vizuális emlékek (limit: {limit})...")

    request = hope_pb2.GetVisualMemoriesRequest(
        limit=limit,
        min_importance=0.0,
        recent_only=True
    )

    try:
        response = stub.GetVisualMemories(request)

        print(f"[IMG] Összesen: {response.total} vizuális emlék")
        for mem in response.memories:
            print(f"\n   [{mem.id[:8]}...]")
            print(f"   Formátum: {mem.format}, Méret: {mem.width}x{mem.height}")
            print(f"   Leírás: {mem.description or 'nincs'}")
            print(f"   Fontosság: {mem.importance:.1f}")

    except grpc.RpcError as e:
        print(f"[X] gRPC hiba: {e.code()} - {e.details()}")


def main():
    """Fő teszt függvény"""
    print("=" * 60)
    print("[EYE]  HOPE VISION TESZT")
    print("=" * 60)

    # gRPC kapcsolat
    print("\n[CONN] Kapcsolódás: localhost:50051...")
    channel = grpc.insecure_channel('localhost:50051')
    vision_stub = hope_pb2_grpc.VisionServiceStub(channel)

    # Kép betöltése vagy generálása
    if len(sys.argv) > 1:
        image_path = sys.argv[1]
        if not os.path.exists(image_path):
            print(f"[X] A fájl nem található: {image_path}")
            sys.exit(1)

        print(f"[FILE] Kép betöltése: {image_path}")
        with open(image_path, "rb") as f:
            image_data = f.read()
        description = os.path.basename(image_path)
    else:
        print("[GEN] Teszt PNG generálása (2x2 pixel)...")
        image_data = create_test_png()
        description = "Generált teszt kép"

    # Tesztek futtatása
    test_see(vision_stub, image_data, description)
    test_vision_status(vision_stub)
    test_visual_memories(vision_stub)

    print("\n" + "=" * 60)
    print("[*] Teszt befejezve! Hope LÁT!")
    print("=" * 60)


if __name__ == "__main__":
    main()
