@echo off
chcp 65001 >nul
title Hope OS - Full Stack

echo ============================================
echo    Hope OS - Teljes Rendszer Inditas
echo    ()=[] - A tiszta potencialbol minden megszuletik
echo ============================================
echo.

cd /d D:\hope-rust

:: 1. Rust gRPC Server
echo [1/7] Rust gRPC Server inditas (port 50051)...
start /B "" target\release\hope.exe serve >nul 2>&1
timeout /t 2 /nobreak >nul

:: 2. TTS Server (Berta - Piper)
echo [2/7] TTS Server inditas (port 8880)...
start /B "" cmd /c "cd /d D:\§§§§§§§§§§§§§§§§§§§§\hope\services\tts && python server.py" >nul 2>&1
timeout /t 2 /nobreak >nul

:: 3. XTTS Streaming Server
echo [3/7] XTTS Streaming Server inditas (port 8881)...
start /B "" cmd /c "cd /d D:\§§§§§§§§§§§§§§§§§§§§\hope\services\tts && python xtts_server.py" >nul 2>&1
timeout /t 2 /nobreak >nul

:: 4. STT Server (Sherpa - Lightning Fast)
echo [4/7] STT Server inditas (port 2022)...
start /B "" cmd /c "cd /d D:\§§§§§§§§§§§§§§§§§§§§\hope\services\stt && python sherpa_stt.py" >nul 2>&1
timeout /t 2 /nobreak >nul

:: 5. PWA Server (Rust Edition)
echo [5/7] PWA Server inditas (port 8766)...
start /B "" cmd /c "cd /d D:\hope-rust && python_client\venv\Scripts\python.exe pwa_server.py" >nul 2>&1
timeout /t 2 /nobreak >nul

:: 6. Brain
echo [6/7] Brain inditas...
start /B "" cmd /c "cd /d D:\hope-rust\python_client && venv\Scripts\python.exe brain.py" >nul 2>&1
timeout /t 2 /nobreak >nul

:: 7. Ngrok
echo [7/7] Ngrok tunnel inditas (port 8766)...
start /B "" ngrok http 8766 >nul 2>&1

echo.
echo ============================================
echo    Hope OS elindult!
echo.
echo    Szolgaltatasok:
echo    - Server: gRPC @ localhost:50051
echo    - TTS:    Berta @ localhost:8880
echo    - XTTS:   Streaming @ localhost:8881
echo    - STT:    Whisper @ localhost:2022
echo    - PWA:    http://127.0.0.1:8766
echo    - Brain:  fut
echo    - Ngrok:  http://127.0.0.1:4040
echo.
echo    Nyisd meg: http://127.0.0.1:8766
echo ============================================
echo.
echo Nyomj ENTER-t a LEALLITASHOZ...
pause >nul

echo.
echo Leallitas...
taskkill /F /IM hope.exe >nul 2>&1
taskkill /F /IM ngrok.exe >nul 2>&1
taskkill /F /FI "WINDOWTITLE eq Hope*" >nul 2>&1
echo Kesz!
