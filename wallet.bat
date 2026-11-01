@echo off
title Blackchain Post-Quantum Wallet

:: Add .cargo\bin to PATH if present
if exist "%USERPROFILE%\.cargo\bin" (
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)

:: Run pre-compiled wallet binary if present
if exist "target\release\blackchain-wallet.exe" (
    target\release\blackchain-wallet.exe %*
    goto end
)

cargo run --release --bin blackchain-wallet -- %*

:end
