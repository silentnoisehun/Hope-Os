#!/usr/bin/env python3
"""
Proto fájlok újragenerálása Python-hoz.

Használat:
    python regenerate_proto.py
"""

import subprocess
import sys
import os

def main():
    # Váltás a script könyvtárába
    script_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(script_dir)

    proto_path = os.path.join(script_dir, "..", "proto")
    proto_file = os.path.join(proto_path, "hope.proto")

    if not os.path.exists(proto_file):
        print(f"[X] Proto fájl nem található: {proto_file}")
        sys.exit(1)

    print("[*] Proto fájlok generálása...")

    cmd = [
        sys.executable, "-m", "grpc_tools.protoc",
        f"-I{proto_path}",
        f"--python_out={script_dir}",
        f"--grpc_python_out={script_dir}",
        proto_file
    ]

    try:
        result = subprocess.run(cmd, capture_output=True, text=True)

        if result.returncode == 0:
            print("[OK] Sikeres! Generált fájlok:")
            print("   - hope_pb2.py")
            print("   - hope_pb2_grpc.py")
        else:
            print(f"[X] Hiba: {result.stderr}")
            sys.exit(1)

    except FileNotFoundError:
        print("[X] grpcio-tools nincs telepítve!")
        print("Telepítsd: pip install grpcio grpcio-tools")
        sys.exit(1)


if __name__ == "__main__":
    main()
