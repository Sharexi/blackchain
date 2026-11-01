@echo off
title Blackchain Post-Quantum Node
echo ============================================================
echo      Starting Blackchain Inference-PoUW Node Daemon         
echo ============================================================

:: Add .cargo\bin to PATH if present
if exist "%USERPROFILE%\.cargo\bin" (
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)

:: If the compiled release binary exists, run it directly (no Cargo needed!)
if exist "target\release\blackchain-node.exe" (
    target\release\blackchain-node.exe %*
    goto end
)

:: Otherwise fallback to cargo
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [!] Errore: 'cargo' non trovato nel PATH.
    echo Riavvia il terminale oppure esegui il file compilato in target\release\blackchain-node.exe
    pause
    exit /b 1
)

cargo run --release --bin blackchain-node -- %*

:end
pause
