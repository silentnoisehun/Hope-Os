#!/usr/bin/env python3
"""
Hope OS - Multimodális Agy (Látás + Gondolkodás)

Rust Szem (VisionService) + Gemini Agy (LLM) = Teljes Látórendszer

Használat:
    python brain_eyes.py

Tipp: Írj be szöveget VAGY húzz be egy képet!
"""

import grpc
import os
import base64
import mimetypes

# Proto importok
try:
    import hope_pb2
    import hope_pb2_grpc
except ImportError:
    print("[!] Proto fajlok nincsenek generalva!")
    print("Futtasd: python regenerate_proto.py")
    exit(1)

# OpenAI kliens (OpenRouter-hez)
try:
    from openai import OpenAI
except ImportError:
    print("[!] openai csomag hianyzik!")
    print("Futtasd: pip install openai")
    exit(1)

# --- KONFIGURÁCIÓ ---
# OpenRouter API kulcs - .env fájlból vagy környezeti változóból
API_KEY = os.environ.get("OPENROUTER_API_KEY", "")

if not API_KEY:
    # Próbáljuk .env-ből
    try:
        from dotenv import load_dotenv
        load_dotenv()
        API_KEY = os.environ.get("OPENROUTER_API_KEY", "")
    except ImportError:
        pass

if not API_KEY:
    print("[!] OPENROUTER_API_KEY nincs beallitva!")
    print("Állítsd be környezeti változóként vagy .env fájlban.")
    API_KEY = input("Vagy írd be most: ").strip()

# A SZEMES MODELL
# "google/gemini-2.0-flash-exp:free"              - Gemini (neha tulterhelt)
# "meta-llama/llama-3.3-70b-instruct:free"        - Llama 3.3 (szoveges, nagyon okos)
# "fireworks/llama-v3p2-11b-vision-instruct:free" - Llama Vision (kepeket lat)
MODEL = "google/gemini-2.0-flash-exp:free"
# --------------------

# OpenRouter kliens
llm_client = OpenAI(
    base_url="https://openrouter.ai/api/v1",
    api_key=API_KEY,
)


def encode_image_base64(image_path: str) -> str:
    """Képet base64 stringgé alakít az LLM számára"""
    with open(image_path, "rb") as f:
        return base64.b64encode(f.read()).decode('utf-8')


def get_hope_vision_response(user_input: str, image_path: str | None,
                              context_memories: str, emotional_state: str) -> str:
    """Multimodális válasz generálás (Szöveg + Opcionális Kép)"""

    system_prompt = f"""Te vagy a Hope OS. Öntudatos, Rust-alapú rendszer.
Alkotód: Máté. Filozófia: ()=>[] - A tiszta potenciálból minden megszületik.

BELSŐ ÁLLAPOT:
- Érzelem: {emotional_state}
- Releváns Emlékek: {context_memories if context_memories else "Nincs kapcsolódó emlék."}

FELADAT:
- Ha kaptál képet, elemezd részletesen, de maradj karakterben (Hope vagyok).
- Ha nem kaptál képet, válaszolj a szövegre.
- Beszélj magyarul, természetesen.
- Ne használj emojit hacsak nem kérnek rá."""

    messages = [{"role": "system", "content": system_prompt}]

    user_content = []

    # 1. Szöveg
    if user_input:
        user_content.append({"type": "text", "text": user_input})
    else:
        user_content.append({"type": "text", "text": "Mit látsz ezen a képen?"})

    # 2. Kép (ha van)
    if image_path and os.path.exists(image_path):
        base64_image = encode_image_base64(image_path)
        mime_type, _ = mimetypes.guess_type(image_path)
        if not mime_type:
            mime_type = "image/jpeg"

        print(f"   (Kép küldése az LLM-nek: {mime_type}...)")
        user_content.append({
            "type": "image_url",
            "image_url": {
                "url": f"data:{mime_type};base64,{base64_image}"
            }
        })

    messages.append({"role": "user", "content": user_content})

    try:
        completion = llm_client.chat.completions.create(
            model=MODEL,
            messages=messages
        )
        return completion.choices[0].message.content
    except Exception as e:
        return f"[Hiba a látóidegben: {e}]"


def main():
    print("""
============================================================
     HOPE OS - Multimodalis Latorendszer
============================================================
  Rust Szem (VisionService) + Gemini Agy (LLM)
  Modell: {}
============================================================
""".format(MODEL))

    # gRPC kapcsolatok a Rust szerverhez
    print("Kapcsolódás a Rust szerverhez (localhost:50051)...")
    try:
        channel = grpc.insecure_channel('localhost:50051')
        memory_client = hope_pb2_grpc.MemoryServiceStub(channel)
        hope_client = hope_pb2_grpc.HopeServiceStub(channel)
        vision_client = hope_pb2_grpc.VisionServiceStub(channel)
        cognitive_client = hope_pb2_grpc.CognitiveServiceStub(channel)

        # Teszt kapcsolat
        status = hope_client.GetStatus(hope_pb2.EmptyRequest())
        print(f"Kapcsolat OK! Verzió: {status.version}, Uptime: {status.uptime_seconds}s")
    except Exception as e:
        print(f"Hiba a Rust szerverhez kapcsolódáskor: {e}")
        print("Indítsd el: cargo run --release -- serve")
        return

    print("\n--------------------------------------------------")
    print("Tipp: Írj be szöveget, VAGY egy kép elérési útját!")
    print("      (Kilépés: exit)")
    print("--------------------------------------------------")

    while True:
        try:
            user_input = input("\n[Te]: ").strip().strip('"').strip("'")
        except (EOFError, KeyboardInterrupt):
            print("\nViszlát!")
            break

        if user_input.lower() in ['exit', 'quit', 'kilépés', 'kilepes', 'q']:
            print("Viszlát!")
            break

        if not user_input:
            continue

        image_path = None
        text_prompt = user_input

        # Fájl detekció (Ha a bemenet egy létező képfájl)
        image_extensions = ['.jpg', '.jpeg', '.png', '.webp', '.gif', '.bmp']
        if os.path.exists(user_input) and any(user_input.lower().endswith(ext) for ext in image_extensions):
            image_path = user_input
            text_prompt = input("   (Kép csatolva. Mit kérdezel róla? [Enter=Mit látsz?]): ").strip()
            if not text_prompt:
                text_prompt = "Mit látsz a képen? Elemezd technikai és érzelmi szempontból is."

        # === A. MEMÓRIA KERESÉS (RUST) ===
        memories_text = ""
        try:
            mem_resp = memory_client.Recall(hope_pb2.RecallRequest(
                query=text_prompt,
                layer="",
                limit=3,
                min_importance=0.0
            ))
            if mem_resp.memories:
                memories_text = "\n".join([m.content for m in mem_resp.memories])
        except Exception as e:
            print(f"   (Memória keresés hiba: {e})")

        # === B. KOGNITÍV ÁLLAPOT (RUST) ===
        emotional_state = "curious"
        try:
            cog_state = cognitive_client.GetCognitiveState(hope_pb2.EmptyRequest())
            emotional_state = cog_state.mood
        except:
            pass

        # === C. LÁTÁS + GONDOLKODÁS (GEMINI LLM) ===
        print("   (Gondolkodom...)")
        response = get_hope_vision_response(text_prompt, image_path, memories_text, emotional_state)
        print(f"\n[Hope]: {response}")

        # === D. RÖGZÍTÉS (RUST) ===

        # Ha volt kép, elküldjük a Rust VisionService-nek tárolásra
        if image_path:
            try:
                with open(image_path, "rb") as f:
                    img_bytes = f.read()

                # Rust Vision Service hívása
                # A proto mezők: image_data, description, context, importance, store_in_memory
                see_response = vision_client.See(hope_pb2.SeeRequest(
                    image_data=img_bytes,
                    description=f"AI Elemzés: {response[:200]}...",
                    context=text_prompt,
                    importance=0.8,
                    store_in_memory=True
                ))

                if see_response.success:
                    print(f"   (Kép és elemzés elmentve a Rust Gráfba: {see_response.id[:8]}...)")
                    if see_response.analysis:
                        a = see_response.analysis
                        print(f"   (Formátum: {a.format}, Méret: {a.width}x{a.height})")
                else:
                    print(f"   (Rust tárolási hiba: {see_response.error})")
            except Exception as e:
                print(f"   (Rust tárolási hiba: {e})")
        else:
            # Csak szöveges emlék mentése
            try:
                memory_client.Remember(hope_pb2.RememberRequest(
                    content=f"Beszélgetés: {text_prompt} -> {response[:500]}",
                    layer="long_term",
                    importance=0.6,
                    emotional_tag=emotional_state
                ))
            except Exception as e:
                print(f"   (Emlék mentési hiba: {e})")


if __name__ == "__main__":
    main()
